use crate::{board::Board, mcts::Mcts, moves::Move, search::Minimax};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EngineType {
    Minimax,
    MCTS
}

pub struct Engine {
    engine_type: EngineType,
    minimax: Option<Minimax>,
    mcts: Option<Mcts>
}

impl Engine {
    pub fn new(engine_type: EngineType) -> Engine {
        Engine {
            engine_type,
            minimax: if engine_type == EngineType::Minimax { Some(Minimax::new()) } else { None },
            mcts: if engine_type == EngineType::MCTS { Some(Mcts::new()) } else { None }
        }
    }

    pub fn search(&mut self, board: &mut Board, depth: Option<u8>, time_limit: Option<u64>) -> Move {
        match self.engine_type {
            EngineType::Minimax => {
                let engine = self.minimax.as_mut().unwrap();
                engine.search(board, depth.unwrap_or(7), f64::NEG_INFINITY, f64::INFINITY, true).moves.first().unwrap().clone()
            },
            EngineType::MCTS => {
                let engine = self.mcts.as_mut().unwrap();
                engine.search(board, time_limit.unwrap_or(10000))
            }
        }
    }

    pub fn iterative_deepening(&mut self, board: &mut Board, depth: u8, time_limit: u64) -> Option<Move> {
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
}