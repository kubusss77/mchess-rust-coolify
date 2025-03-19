use crate::r#const::{CASTLING_VALUE, CHECK_VALUE, KILLER_MOVE_VALUE, PROMOTION_VALUE, PV_MOVE};
use crate::evaluation::{evaluate, evaluate_move, EvaluationResult};
use crate::board::{Board, ResultType};
use crate::moves::{Move, MoveType};
use core::f64;
use std::collections::HashMap;

pub struct Chess {
    evaluation_cache: HashMap<i64, EvaluationResult>,
    move_evaluation_cache: HashMap<usize, f64>,
    transposition_table: HashMap<i64, Node>,
    killer_moves: Vec<Vec<Option<Move>>>
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeType {
    PV,
    Cut,
    All
}

#[derive(Debug, Clone)]
pub struct Node {
    depth: u8,
    node_type: NodeType,
    score: f64,
    best_move: Option<Move>
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
            move_evaluation_cache: HashMap::new(),
            transposition_table: HashMap::new(),
            killer_moves: vec![vec![None; 2]; 100]
        }
    }

    pub fn store_position(&mut self, board: &Board, depth: u8, node_type: NodeType, score: f64, best_move: Option<Move>) {
        let node = Node {
            depth,
            node_type,
            score,
            best_move
        };

        self.transposition_table.insert(board.hash, node);
    }

    pub fn check_position(&self, board: &Board, depth: u8, alpha: f64, beta: f64) -> Option<(f64, Option<Move>)> {
        if let Some(node) = self.transposition_table.get(&board.hash) {
            if node.depth >= depth {
                match node.node_type {
                    NodeType::PV => return Some((node.score, node.best_move.clone())),
                    NodeType::Cut if node.score >= beta => return Some((beta, node.best_move.clone())),
                    NodeType::All if node.score <= alpha => return Some((alpha, node.best_move.clone())),
                    _ => {}
                }
            }
        }

        None
    }

    pub fn store_killer_move(&mut self, m: &Move, depth: u8) {
        let first_killer = &self.killer_moves[depth as usize][0];

        if let Some(killer) = first_killer {
            if killer != m {
                self.killer_moves[depth as usize][1] = Some(killer.clone());
                self.killer_moves[depth as usize][0] = Some(m.clone());
            }
        }
    }

    pub fn search(&mut self, board: &mut Board, depth: u8, _alpha: f64, _beta: f64, maximizer: bool) -> SearchResult {
        if board.get_result() != ResultType::None || depth == 0 {
            let evaluation = self.evaluate(board);
            return SearchResult {
                value: evaluation.to_value(),
                moves: vec![]
            }
        }

        let mut alpha = _alpha;
        let mut beta = _beta;

        if let Some((value, m)) = self.check_position(board, depth, alpha, beta) {
            if m.is_some() {
                return SearchResult {
                    value,
                    moves: if let Some(m_) = m { vec![m_] } else { vec![] }
                }
            }
        }

        if maximizer {
            let mut value = f64::NEG_INFINITY;
            let mut moves: Vec<Move> = vec![];
            let mut best_move = None;
            let mut node_type = NodeType::All;

            let legal_moves = self.sort(board.get_total_legal_moves(None), board, depth);

            for m in legal_moves {
                let history = board.make_move(&m);

                let result = self.search(board, depth - 1, alpha, beta, false);

                board.unmake_move(&m, &history);

                let old_value = value;
                value = value.max(result.value);

                if old_value < value {
                    best_move = Some(m.clone());
                    let mut new_moves = vec![m.clone()];
                    new_moves.extend(result.moves);
                    moves = new_moves;
                }

                if value > alpha {
                    alpha = value;
                    node_type = NodeType::PV;
                }

                if beta <= alpha {
                    self.store_killer_move(&m, depth);

                    node_type = NodeType::Cut;
                    break
                }
            }

            self.store_position(board, depth, node_type, value, best_move);

            SearchResult {
                value,
                moves
            }
        } else {
            let mut value = f64::INFINITY;
            let mut moves: Vec<Move> = vec![];
            let mut best_move = None;
            let mut node_type = NodeType::All;

            let legal_moves = self.sort(board.get_total_legal_moves(None), board, depth);
            
            for m in legal_moves {
                let history = board.make_move(&m);

                let result = self.search(board, depth - 1, alpha, beta, true);

                board.unmake_move(&m, &history);
                let old_value = value;
                value = value.min(result.value);

                if old_value > value {
                    best_move = Some(m.clone());
                    let mut new_moves = vec![m.clone()];
                    new_moves.extend(result.moves);
                    moves = new_moves;
                }

                if value < beta {
                    node_type = NodeType::PV;
                    beta = value;
                }

                if beta <= alpha {
                    self.store_killer_move(&m, depth);

                    node_type = NodeType::Cut;
                    break
                }
            }

            self.store_position(board, depth, node_type, value, best_move);

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

    pub fn evaluate_move(&mut self, m: &Move, board: &mut Board, depth: u8) -> f64 {
        if self.move_evaluation_cache.contains_key(&m.hash()) {
            return *self.move_evaluation_cache.get(&m.hash()).unwrap()
        }
        let mut value = 0.0;

        if let Some(node) = self.transposition_table.get(&board.hash) {
            if let Some(best_move) = &node.best_move {
                if best_move == m {
                    value += PV_MOVE;
                }
            }
        }

        value += m.mvv_lva();

        if !m.move_type.contains(&MoveType::Capture) {
            if let Some(killer) = &self.killer_moves[depth as usize][0] {
                if m == killer {
                    value += KILLER_MOVE_VALUE;
                }
            }

            if let Some(killer) = &self.killer_moves[depth as usize][1] {
                if m == killer {
                    value += KILLER_MOVE_VALUE - 1000.0;
                }
            }
        }

        if m.move_type.contains(&MoveType::Promotion) {
            value += PROMOTION_VALUE;
        }

        if m.move_type.contains(&MoveType::Check) {
            value += CHECK_VALUE;
        }

        if m.move_type.contains(&MoveType::Castling) {
            value += CASTLING_VALUE;
        }

        self.move_evaluation_cache.insert(m.hash(), value);

        value
    }

    pub fn sort(&mut self, moves: Vec<Move>, board: &mut Board, depth: u8) -> Vec<Move> {
        let scores = moves.iter()
            .map(|m| self.evaluate_move(m, board, depth));
        
        let mut indices: Vec<(usize, f64)> = scores
            .enumerate()
            .map(|(i, score)| (i, score))
            .collect();

        indices.sort_by(|(_, a), (_, b)| b.total_cmp(a));

        let mut result: Vec<Move> = Vec::with_capacity(moves.len());

        for (i, _) in indices {
            result.push(moves[i].clone());
        }

        if depth == 6 {
            println!("{:?}", result);
        }
        
        result
    }
}