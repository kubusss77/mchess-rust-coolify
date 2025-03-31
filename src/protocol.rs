use std::{io::{self, Write}, path::Path};

use crate::{board::Board, engine::{Engine, EngineType}, moves::{Move, MoveType}, piece::{PieceColor, PieceType}};

pub struct UciProtocol {
    engine: Engine,
    board: Board,
    engine_type: EngineType,
    enable_book: bool,
    move_history: Vec<String>
}

impl UciProtocol {
    pub fn new() -> Self {
        UciProtocol { 
            engine: Engine::new(EngineType::Minimax, false), 
            board: Board::startpos(),
            engine_type: EngineType::Minimax, // default
            enable_book: false,
            move_history: vec![]
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        self.identify();

        let stdin = io::stdin();
        let mut input = String::new();

        self.engine.load_book(Path::new("book.pgn"))?;

        if let Some(book) = self.engine.book.as_ref() {
            book.print_statistics();
        }

        loop {
            input.clear();
            stdin.read_line(&mut input)?;
            let command = input.trim();

            match command {
                "quit" => break,
                "uci" => self.identify(),
                "isready" => println!("readyok"),
                cmd if cmd.starts_with("position") => self.handle_position(cmd),
                cmd if cmd.starts_with("go") => self.handle_go(cmd),
                cmd if cmd.starts_with("setoption") => self.set_option(cmd),
                "ucinewgame" => {
                    self.board = Board::startpos();
                    self.engine.switch_to(self.engine_type);
                    self.engine.set_book_enabled(self.enable_book);
                },
                "stop" => {
                    self.engine.stop();
                },
                a => println!("info string Unknown option {}", a)
            }

            io::stdout().flush().unwrap();
        }

        Ok(())
    }

    pub fn identify(&mut self) {
        println!("id name mchess");
        println!("id author ggod");
        println!("option name EngineType type combo default Minimax var Minimax var MCTS");
        println!("option name EnableBook type check default false");
        println!("uciok");
    }

    fn set_option(&mut self, command: &str) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let name_index = parts.iter().position(|&p| p.to_lowercase() == "name");
        let value_index = parts.iter().position(|&p| p.to_lowercase() == "value");

        if name_index.is_none() {
            return;
        }

        let name_start = name_index.unwrap() + 1;
        let name_end = value_index.unwrap_or(parts.len());
        let name = parts[name_start..name_end].join(" ").to_lowercase();

        let value = if let Some(index) = value_index {
            if index + 1 < parts.len() {
                parts[(index + 1)..].join("")
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        match name.as_str() {
            "enginetype" | "engine type" => {
                match value.to_lowercase().as_str() {
                    "minimax" | "alphabeta" | "default" => {
                        println!("info string Setting engine type to Minimax");
                        self.engine_type = EngineType::Minimax;
                        self.engine.switch_to(self.engine_type);
                        self.engine.set_book_enabled(self.enable_book);
                    },
                    "mcts" => {
                        println!("info string Setting engine type to MCTS");
                        self.engine_type = EngineType::MCTS;
                        self.engine.switch_to(self.engine_type);
                        self.engine.set_book_enabled(self.enable_book);
                    },
                    a => println!("info string Unknown engine type: {}, current: {:?}", a, self.engine_type)
                }
            },
            "enablebook" | "enable book" => {
                match value.to_lowercase().as_str() {
                    "true" => {
                        println!("info string Setting enable book to true");
                        self.enable_book = true;
                        self.engine.set_book_enabled(true);
                    },
                    "false" => {
                        println!("info string Setting enable book to false");
                        self.enable_book = false;
                        self.engine.set_book_enabled(false);
                    },
                    a => println!("info string Unknown enable book option: {}, current: {:?}", a, self.engine_type)
                }
            },
            a => println!("info string Unknown option: {}", a)
        }
    }

    fn handle_position(&mut self, command: &str) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let pos_type = parts.get(1).unwrap_or(&"");

        match *pos_type {
            "startpos" => {
                self.board = Board::startpos();
                self.engine.switch_to(self.engine_type);
                self.engine.set_book_enabled(self.enable_book);

                if let Some(moves_index) = parts.iter().position(|&p| p == "moves") {
                    self.move_history.clear();
                    for i in (moves_index + 1)..parts.len() {
                        let uci_move = parts[i];
                        println!("info String {uci_move}");
                        self.move_uci(uci_move.trim());
                    }
                }
            },
            "fen" => {
                if parts.len() >= 8 {
                    let fen = parts[2..8].join(" ");
                    self.board = Board::from_fen(&fen);

                    if let Some(moves_index) = parts.iter().position(|&p| p == "moves") {
                        self.move_history.clear();
                        for i in (moves_index + 1)..parts.len() {
                            let uci_move = parts[i];
                            self.move_uci(uci_move.trim());
                        }
                    }
                }
            },
            _ => {}
        }
    }

    fn handle_go(&mut self, command: &str) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let mut depth = 5;
        let mut time_limit = 5000;
        let mut wtime = None;
        let mut btime = None;
        let mut winc = None;
        let mut binc = None;
        let mut movestogo = None;
        let mut movetime = None;

        for i in 0..parts.len() - 1 {
            if parts[i] == "depth" {
                if let Ok(d) = parts[i + 1].parse::<u8>() {
                    depth = d;
                }
            } else if parts[i] == "wtime" {
                if let Ok(t) = parts[i + 1].parse::<u64>() {
                    wtime = Some(t);
                }
            } else if parts[i] == "btime" {
                if let Ok(t) = parts[i + 1].parse::<u64>() {
                    btime = Some(t);
                }
            } else if parts[i] == "winc" {
                if let Ok(inc) = parts[i + 1].parse::<u64>() {
                    winc = Some(inc);
                }
            } else if parts[i] == "binc" {
                if let Ok(inc) = parts[i + 1].parse::<u64>() {
                    binc = Some(inc);
                }
            } else if parts[i] == "movestogo" {
                if let Ok(mtg) = parts[i + 1].parse::<u32>() {
                    movestogo = Some(mtg);
                }
            } else if parts[i] == "movetime" {
                if let Ok(mt) = parts[i + 1].parse::<u64>() {
                    movetime = Some(mt);
                }
            }
        }

        if let Some(mt) = movetime {
            time_limit = mt;
        } else if wtime.is_some() || btime.is_some() {
            let is_white = self.board.turn == PieceColor::White;
            let time = if is_white { wtime } else { btime };
            let inc = if is_white { winc } else { binc };

            if let Some(remaining) = time {
                let moves_left = movestogo.unwrap_or(30);
                let increment = inc.unwrap_or(0);

                let base_time = remaining / moves_left as u64;
                let allocated = base_time + increment / 2;

                time_limit = std::cmp::min(allocated, remaining / 5);
            }
        }

        let result = self.engine.iterative_deepening(&mut self.board, depth, time_limit, &self.move_history);

        if let Some(best_move) = result.as_ref() {
            println!("info string turn {:?} move clr {:?}", self.board.turn, best_move.piece_color);
            println!("bestmove {}", self.move_to_uci(best_move));
        } else {
            println!("bestmove 0000");
        }

    }

    fn move_uci(&mut self, uci_move: &str) {
        if uci_move.len() < 4 {
            return;
        }

        println!("info string {uci_move} 2");

        let from_file = (uci_move.chars().nth(0).unwrap() as u8 - b'a') as usize;
        let from_rank = 8 - (uci_move.chars().nth(1).unwrap() as u8 - b'0') as usize;
        let to_file = (uci_move.chars().nth(2).unwrap() as u8 - b'a') as usize;
        let to_rank = 8 - (uci_move.chars().nth(3).unwrap() as u8 - b'0') as usize;

        let legal_moves = self.board.get_total_legal_moves(None);

        println!("info string legal_moves {:?}", legal_moves);
        for m in legal_moves {
            if m.from.x == from_file && m.from.y == from_rank && m.to.x == to_file && m.to.y == to_rank {
                if uci_move.len() > 4 {
                    println!("info string > 4 {uci_move}");
                    if m.move_type.contains(&MoveType::Promotion) {
                        self.board.make_move(&m);
                        self.move_history.push(m.to_san(&self.board));
                        break;
                    }
                } else {
                    println!("info string turn bef {:?}", self.board.turn);
                    self.board.make_move(&m);
                    self.move_history.push(m.to_san(&self.board));
                    println!("info string turn aft {:?}", self.board.turn);
                    break;
                }
            }
        }
    }

    fn move_to_uci(&self, m: &Move) -> String {
        let from_file = ('a' as u8 + m.from.x as u8) as char;
        let from_rank = ('8' as u8 - m.from.y as u8) as char;
        let to_file = ('a' as u8 + m.to.x as u8) as char;
        let to_rank = ('8' as u8 - m.to.y as u8) as char;
        
        let mut uci = format!("{}{}{}{}", from_file, from_rank, to_file, to_rank);

        if let Some(promotion) = &m.promote_to {
            let promotion_char = match promotion {
                PieceType::Queen => 'q',
                PieceType::Rook => 'r',
                PieceType::Bishop => 'b',
                PieceType::Knight => 'n',
                _ => ' '
            };

            if promotion_char != ' ' {
                uci.push(promotion_char);
            }
        }

        uci
    }
}