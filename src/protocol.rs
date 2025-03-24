use core::f64;
use std::io::{self, Write};

use crate::{board::Board, moves::{Move, MoveType}, piece::{PieceColor, PieceType}, search::Chess};

pub struct UciProtocol {
    engine: Chess,
    board: Board
}

impl UciProtocol {
    pub fn new() -> Self {
        UciProtocol { 
            engine: Chess::new(), 
            board: Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1") 
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        println!("id name mchess");
        println!("id author ggod");
        println!("uciok");

        let stdin = io::stdin();
        let mut input = String::new();

        loop {
            input.clear();
            stdin.read_line(&mut input)?;
            let command = input.trim();

            match command {
                "quit" => break,
                "uci" => {
                    println!("id name mchess");
                    println!("id name ggod");
                    println!("uciok");
                },
                "isready" => println!("readyok"),
                cmd if cmd.starts_with("position") => self.handle_position(cmd),
                cmd if cmd.starts_with("go") => self.handle_go(cmd),
                "stop" => {

                },
                _ => {}
            }

            io::stdout().flush().unwrap();
        }

        Ok(())
    }    

    fn handle_position(&mut self, command: &str) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let pos_type = parts.get(1).unwrap_or(&"");

        match *pos_type {
            "startpos" => {
                self.board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

                if let Some(moves_index) = parts.iter().position(|&p| p == "moves") {
                    for i in (moves_index + 1)..parts.len() {
                        let uci_move = parts[i];
                        self.move_uci(uci_move);
                    }
                }
            },
            "fen" => {
                if parts.len() >= 8 {
                    let fen = parts[2..8].join(" ");
                    self.board = Board::from_fen(&fen);

                    if let Some(moves_index) = parts.iter().position(|&p| p == "moves") {
                        for i in (moves_index + 1)..parts.len() {
                            let uci_move = parts[i];
                            self.move_uci(uci_move);
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

        for i in 0..parts.len() - 1 {
            if parts[i] == "depth" {
                if let Ok(d) = parts[i + 1].parse::<u8>() {
                    depth = d;
                }
            }
        }

        let turn = self.board.turn;
        let result = self.engine.search(&mut self.board, depth, f64::NEG_INFINITY, f64::INFINITY, turn == PieceColor::White);

        if let Some(best_move) = result.moves.first() {
            println!("bestmove {}", self.move_to_uci(best_move));
        } else {
            println!("bestmove 0000");
        }

    }

    fn move_uci(&mut self, uci_move: &str) {
        if uci_move.len() < 4 {
            return;
        }

        let from_file = (uci_move.chars().nth(0).unwrap() as u8 - b'a') as usize;
        let from_rank = 8 - (uci_move.chars().nth(1).unwrap() as u8 - b'0') as usize;
        let to_file = (uci_move.chars().nth(2).unwrap() as u8 - b'a') as usize;
        let to_rank = 8 - (uci_move.chars().nth(3).unwrap() as u8 - b'0') as usize;

        let legal_moves = self.board.get_total_legal_moves(None);
        for m in legal_moves {
            if m.from.x == from_file && m.from.y == from_rank && m.to.x == to_file && m.to.y == to_rank {
                if uci_move.len() > 4 {
                    if m.move_type.contains(&MoveType::Promotion) {
                        self.board.make_move(&m);
                        break;
                    }
                } else {
                    self.board.make_move(&m);
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