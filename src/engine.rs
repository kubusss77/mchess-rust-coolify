use std::path::Path;

use crate::{board::Board, book::OpeningBook, mcts::Mcts, moves::Move, search::Minimax};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EngineType {
    Minimax,
    MCTS
}

pub struct Engine {
    engine_type: EngineType,
    minimax: Option<Minimax>,
    mcts: Option<Mcts>,
    pub book: Option<OpeningBook>,
    pub enable_book: bool
}

impl Engine {
    pub fn new(engine_type: EngineType, enable_book: bool) -> Engine {
        Engine {
            engine_type,
            minimax: if engine_type == EngineType::Minimax { Some(Minimax::new()) } else { None },
            mcts: if engine_type == EngineType::MCTS { Some(Mcts::new()) } else { None },
            enable_book,
            book: None
        }
    }

    pub fn switch_to(&mut self, engine_type: EngineType) {
        self.engine_type = engine_type;
        self.minimax = if engine_type == EngineType::Minimax { Some(Minimax::new()) } else { None };
        self.mcts = if engine_type == EngineType::MCTS { Some(Mcts::new()) } else { None };
    }

    pub fn load_book(&mut self, path: &Path) -> std::io::Result<usize> {
        let mut book = OpeningBook::new();

        let loaded_games = if path.is_dir() {
            book.load_book_directory(path)?
        } else {
            book.load_pgn_file(path)?
        };
        
        self.book = Some(book);
        Ok(loaded_games)
    }

    pub fn search(&mut self, board: &mut Board, depth: Option<u8>, time_limit: Option<u64>, move_history: &Vec<String>) -> Option<Move> {
        if self.enable_book {
            if let Some(book) = &self.book {
                if let Some(book_move) = book.get_best_move(&move_history) {
                    println!("book move found {book_move}");
                    return book.to_move(&book_move, board);
                }
            }
        }

        match self.engine_type {
            EngineType::Minimax => {
                let engine = self.minimax.as_mut().unwrap();
                engine.search(board, depth.unwrap_or(7), f64::NEG_INFINITY, f64::INFINITY, true).moves.first().cloned()
            },
            EngineType::MCTS => {
                let engine = self.mcts.as_mut().unwrap();
                engine.search(board, time_limit.unwrap_or(10000))
            }
        }
    }

    pub fn iterative_deepening(&mut self, board: &mut Board, depth: u8, time_limit: u64, move_history: &Vec<String>) -> Option<Move> {
        if self.enable_book {
            if let Some(book) = &self.book {
                if let Some(book_move) = book.get_best_move(&move_history) {
                    println!("book move found {book_move}");
                    return book.to_move(&book_move, board);
                }
            }
        }

        match self.engine_type {
            EngineType::Minimax => {
                let engine = self.minimax.as_mut().unwrap();
                engine.iterative_deepening(board, depth, time_limit).moves.first().cloned()
            },
            EngineType::MCTS => {
                let engine = self.mcts.as_mut().unwrap();
                engine.iterative_deepening(board, depth as u32, time_limit)
            }
        }
    }

    pub fn stop(&mut self) {
        match self.engine_type {
            EngineType::Minimax => {
                let engine = self.minimax.as_mut().unwrap();
                engine.stop();
            },
            EngineType::MCTS => {
                let engine = self.mcts.as_mut().unwrap();
                engine.stop();
            }
        }
    }

    pub fn set_book_enabled(&mut self, enabled: bool) {
        self.enable_book = enabled;
    }
}