use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    routing::get,
    Router,
    response::IntoResponse,
    http::StatusCode,
    Json, extract::State,
    routing::post,
};
use serde::{Deserialize, Serialize};
use std::{io::Write, path::Path, sync::{Arc, Mutex}, time::Duration};
use tokio::{net::TcpListener, time::timeout};
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::env;
use dotenv::dotenv;

use crate::protocol::UciProtocol;

struct AppState {
    protocols: Mutex<HashMap<String, UciProtocol>>,
    template: Mutex<UciProtocol>
}

#[derive(Deserialize)]
struct UciRequest {
    client_id: String,
    command: String
}

#[derive(Serialize)]
struct UciResponse {
    client_id: String,
    response: Vec<String>
}
struct ResponseWriter {
    lines: Vec<String>,
    buffer: String,
}

impl ResponseWriter {
    fn new() -> Self {
        ResponseWriter {
            lines: Vec::new(),
            buffer: String::new(),
        }
    }
    
    fn get_messages(self) -> Vec<String> {
        let mut results = Vec::new();
        let mut info_lines = Vec::new();
        
        for line in self.lines {
            let trimmed = line.trim();
            if trimmed.starts_with("bestmove") {
                if !info_lines.is_empty() {
                    results.push(info_lines.join(" "));
                    info_lines.clear();
                }
                results.push(trimmed.to_string());
            } else if trimmed.starts_with("info") {
                info_lines.push(trimmed.to_string());
            } else if !trimmed.is_empty() {
                results.push(trimmed.to_string());
            }
        }
        
        if !info_lines.is_empty() {
            results.push(info_lines.join(" "));
        }
        
        results
    }
}

impl std::io::Write for ResponseWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Ok(s) = std::str::from_utf8(buf) {
            self.buffer.push_str(s);
            
            if self.buffer.contains('\n') {
                let parts: Vec<&str> = self.buffer.split('\n').collect();
                
                for i in 0..parts.len()-1 {
                    if !parts[i].trim().is_empty() {
                        self.lines.push(parts[i].to_string());
                    }
                }
                
                self.buffer = parts[parts.len()-1].to_string();
            }
        }
        Ok(buf.len())
    }
    
    fn flush(&mut self) -> std::io::Result<()> {
        if !self.buffer.trim().is_empty() {
            self.lines.push(self.buffer.trim().to_string());
            self.buffer.clear();
        }
        Ok(())
    }
}

async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| connection(socket, state))
}

async fn connection(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    let client_id = uuid::Uuid::new_v4().to_string();

    {
        match timeout(Duration::from_secs(5), async {
            let mut protocols = match state.protocols.lock() {
                Ok(p) => p,
                Err(e) => e.into_inner(),
            };
            
            let template = match state.template.lock() {
                Ok(t) => t,
                Err(e) => e.into_inner(),
            };

            let mut new_protocol = UciProtocol::new();
            new_protocol.engine.set_book_enabled(true);

            if let Some(book) = template.engine.book.as_ref() {
                new_protocol.engine.book = Some(book.clone());
            }

            protocols.insert(client_id.clone(), new_protocol);
        }).await {
            Ok(_) => {},
            Err(_) => {
                eprintln!("Timeout initializing client {}", client_id);
                return;
            }
        }
    }

    println!("connection established with {}", client_id);

    let _ = sender.send(Message::Text(format!("established:{}", client_id).into())).await;

    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            if text.trim().is_empty() {
                continue;
            }

            let state = Arc::clone(&state);
            let client_id_clone = client_id.clone();
            let text = text.clone();
            
            let responses = match timeout(Duration::from_secs(30), 
                tokio::task::spawn(async move {
                    process_command(&state, &client_id_clone, &text).await
                })
            ).await {
                Ok(Ok(responses)) => responses,
                Ok(Err(e)) => {
                    eprintln!("Task error for client {}: {:?}", client_id, e);
                    vec!["info string Internal server error".to_string()]
                },
                Err(_) => {
                    eprintln!("Command timed out for client {}", client_id);
                    vec!["info string Processing timed out".to_string()]
                }
            };

            for response in responses {
                if let Err(_) = sender.send(Message::Text(response.into())).await {
                    break;
                }
            }
        } else if let Message::Close(_) = msg {
            break;
        }
    }

    match timeout(Duration::from_secs(5), async {
        let mut protocols = match state.protocols.lock() {
            Ok(p) => p,
            Err(e) => e.into_inner(),
        };
        protocols.remove(&client_id);
    }).await {
        Ok(_) => {},
        Err(_) => eprintln!("Timeout removing client {}", client_id),
    }
}

async fn process_command(state: &Arc<AppState>, client_id: &str, command: &str) -> Vec<String> {
    let protocols_result: Result<std::sync::MutexGuard<'_, HashMap<String, UciProtocol>>, _> = match timeout(Duration::from_secs(5), async {
        match state.protocols.lock() {
            Ok(protocols) => Ok::<_, std::sync::PoisonError<std::sync::MutexGuard<'_, HashMap<String, UciProtocol>>>>(protocols),
            Err(poisoned) => {
                eprintln!("Recovered from poisoned protocols for {}", client_id);
                Ok(poisoned.into_inner())
            }
        }
    }).await {
        Ok(result) => result,
        Err(_) => {
            eprintln!("Timeout acquiring lock for client {}", client_id);
            return vec!["info string Server busy, try again later".to_string()];
        }
    };
    
    let mut protocols = match protocols_result {
        Ok(protocols) => protocols,
        Err(_) => return vec!["info string Internal server error".to_string()],
    };
    let protocol = protocols.entry(client_id.to_string()).or_insert_with(UciProtocol::new);

    match command.trim() {
        "uci" => {
            let mut responses = Vec::new();
            responses.push("id name mchess".to_string());
            responses.push("id author ggod".to_string());
            responses.push("option name EngineType type combo default Minimax var Minimax var MCTS".to_string());
            responses.push("option name EnableBook type check default false".to_string());
            responses.push("uciok".to_string());
            return responses;
        },
        "isready" => {
            return vec!["readyok".to_string()];
        },
        "ucinewgame" => {
            let book = protocol.engine.book.clone();
            let enable_book = protocol.engine.enable_book;
            *protocol = UciProtocol::new();
            protocol.engine.book = book;
            protocol.engine.set_book_enabled(enable_book);
            return vec!["ok".to_string()];
        },
        "stop" => {
            protocol.engine.stop();
            return vec!["ok".to_string()];
        },
        cmd if cmd.starts_with("position") => {
            let mut writer = ResponseWriter::new();

            if let Err(e) = protocol.handle_position(cmd, &mut writer) {
                return vec![format!("info string Error executing position command: {}", e)];
            }
            
            writer.flush().unwrap();

            return writer.get_messages();
        },
        cmd if cmd.starts_with("go") => {
            let mut writer = ResponseWriter::new();
            
            if let Err(e) = protocol.handle_go(cmd, &mut writer) {
                return vec![format!("info string Error executing go command: {}", e)];
            }
            
            writer.flush().unwrap();

            return writer.get_messages();
        },
        cmd if cmd.starts_with("setoption") => {
            let mut writer = ResponseWriter::new();
            
            if let Err(e) = protocol.set_option(cmd, &mut writer) {
                return vec![format!("info string Error executing setoption command: {}", e)];
            }

            writer.flush().unwrap();

            return writer.get_messages();
        },
        "quit" => {
            return vec!["Disconnecting".to_string()];
        },
        _ => {
            return vec![format!("info string Unknown command: {}", command)];
        }
    }
}

async fn command(State(state): State<Arc<AppState>>, Json(request): Json<UciRequest>) -> Result<Json<UciResponse>, (StatusCode, String)> {
    let response = process_command(&state, &request.client_id, &request.command).await;

    Ok(Json(UciResponse {
        client_id: request.client_id,
        response
    }))
}

pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let mut template = UciProtocol::new();
    let book_path = env::var("BOOK_PATH").unwrap_or_else(|_| "book".to_string());
    println!("Loading opening books from {}", book_path);

    let path = Path::new(&book_path);

    template.engine.set_book_enabled(true);

    match template.engine.load_book(path) {
        Ok(_) => println!("Opening book loaded successfully"),
        Err(e) => eprintln!("Failed to load opening book: {}", e),
    }

    let state = Arc::new(AppState {
        protocols: Mutex::new(HashMap::new()),
        template: Mutex::new(template)
    });

    let app = Router::new()
        .route("", get(websocket_handler))
        .route("/uci", post(command))
        .with_state(state);

    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "3100".to_string());
    
    let address = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&address).await?;
    println!("Chess engine server listening on {}", address);
    axum::serve(listener, app).await?;

    Ok(())
}