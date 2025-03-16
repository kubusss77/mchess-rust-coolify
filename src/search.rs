use crate::evaluation::{evaluate, evaluate_move, EvaluationResult};
use crate::board::{Board, ResultType};
use crate::moves::Move;
use core::f64;
use std::collections::HashMap;

pub struct Chess {
    evaluation_cache: HashMap<i64, EvaluationResult>,
    move_evaluation_cache: HashMap<Move, f64>
}

#[derive(Debug)]
pub struct SearchResult {
    value: f64,
    moves: Vec<Move>
}

impl Chess {
    pub fn new() -> Self {
        Chess {
            evaluation_cache: HashMap::new(),
            move_evaluation_cache: HashMap::new()
        }
    }

    pub fn search(&mut self, board: &mut Board, depth: usize, _alpha: f64, _beta: f64, maximizer: bool) -> SearchResult {
        if board.get_result() != ResultType::None || depth == 0 {
            let evaluation = self.evaluate(board);
            return SearchResult {
                value: evaluation.to_value(),
                moves: vec![]
            }
        }

        let mut alpha = _alpha;
        let mut beta = _beta;

        if maximizer {
            let mut value = f64::NEG_INFINITY;
            let mut moves: Vec<Move> = vec![];

            let legal_moves = self.sort(board.get_total_legal_moves(None), board);

            for m in legal_moves {
                let result = self.search(&mut board.move_clone(&m), depth - 1, alpha, beta, false);
                let old_value = value;
                value = value.max(result.value);
                alpha = alpha.max(value);
                if old_value < value {
                    let mut new_moves = vec![m];
                    new_moves.extend(result.moves);
                    moves = new_moves;
                }
                if beta <= alpha {
                    break
                }
            }

            SearchResult {
                value,
                moves
            }
        } else {
            let mut value = f64::INFINITY;
            let mut moves: Vec<Move> = vec![];

            let legal_moves = self.sort(board.get_total_legal_moves(None), board);
            
            for m in legal_moves {
                let result = self.search(&mut board.move_clone(&m), depth - 1, alpha, beta, true);
                let old_value = value;
                value = value.min(result.value);
                beta = beta.min(value);
                if old_value > value {
                    let mut new_moves = vec![m];
                    new_moves.extend(result.moves);
                    moves = new_moves;
                }
                if beta <= alpha {
                    break
                }
            }

            SearchResult {
                value,
                moves
            }
        }
    }

    pub fn evaluate(&mut self, board: &mut Board) -> EvaluationResult {
        if self.evaluation_cache.contains_key(&board.hash) {
            return *self.evaluation_cache.get(&board.hash).unwrap()
        }
        let value = evaluate(board);
        self.evaluation_cache.insert(board.hash, value);

        value
    }

    pub fn evaluate_move(&mut self, m: &Move, board: &mut Board) -> f64 {
        if self.move_evaluation_cache.contains_key(&m) {
            return *self.move_evaluation_cache.get(&m).unwrap()
        }
        let value = evaluate_move(m, board);

        self.move_evaluation_cache.insert(m.clone(), value);

        value
    }

    pub fn sort(&mut self, moves: Vec<Move>, board: &mut Board) -> Vec<Move> {
        let mut clone = moves.clone();
        clone.sort_by(|a: &Move, b: &Move| {
            self.evaluate_move(b, board).total_cmp(&self.evaluate_move(a, board))
        });

        clone
    }
}