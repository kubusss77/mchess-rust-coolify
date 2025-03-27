use std::{collections::HashMap, time::{Duration, Instant}};
use rand::seq::IndexedRandom;

use crate::{board::{Board, ResultType}, r#const::MCTS_MAX_PLIES, evaluation::evaluate, moves::Move, piece::PieceColor};

#[derive(Debug)]
struct Node {
    pub m: Option<Move>,
    pub visits: u32,
    pub score: f64,
    pub children: Vec<Node>,
    pub expanded: bool
}

impl Node {
    fn new(m: Option<Move>) -> Self {
        Node {
            m,
            visits: 0,
            score: 0.0,
            children: Vec::new(),
            expanded: false
        }
    }

    fn get_uct(&self, parent_visits: u32, exp: f64) -> f64 {
        if self.visits == 0 { return f64::INFINITY; }

        let exploitation = self.score / self.visits as f64;
        let exploration = exp * ((parent_visits as f64).ln() / self.visits as f64).sqrt();

        exploitation + exploration
    }
}

pub struct Mcts {
    pub time_limit: u64,
    pub exp: f64,
    pub max_iterations: usize,
    pub nodes_visited: usize,
    node_cache: HashMap<u64, Node>,
    is_stopping: bool
}

impl Mcts {
    pub fn new() -> Self {
        Mcts {
            time_limit: 1000,
            exp: 1.414,
            max_iterations: 10000,
            nodes_visited: 0,
            node_cache: HashMap::new(),
            is_stopping: false
        }
    }

    pub fn search(&mut self, board: &mut Board, time_limit_ms: u64) -> Move {
        self.time_limit = time_limit_ms;
        self.nodes_visited = 0;
        let start_time = Instant::now();
        let time_limit = Duration::from_millis(time_limit_ms);

        let mut root = Node::new(None);
        let mut iterations = 0;

        while start_time.elapsed() < time_limit && !self.is_stopping {
            let mut board_clone = board.clone();
            let path = self.select_and_expand(&mut root, &mut board_clone);
            let result = self.simulate(&mut board_clone);
            self.backpropagate(&mut root, &path, result);

            iterations += 1;
        }

        let best_child = root.children.iter()
            .max_by_key(|child| child.visits)
            .expect("No moves found");

        println!("MCTS completed {} iterations in {:?}", iterations, start_time.elapsed());
        println!("Nodes visited: {}", self.nodes_visited);
        
        best_child.m.clone().unwrap()
    }

    fn select_and_expand(&mut self, node: &mut Node, board: &mut Board) -> Vec<usize> {
        let mut path = Vec::new();
        let mut current_node = node;

        while !current_node.children.is_empty() && current_node.expanded && !self.is_stopping {
            let parent_visits = current_node.visits;
            let best_child_index = current_node.children.iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.get_uct(parent_visits, self.exp).partial_cmp(&b.get_uct(parent_visits, self.exp)).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(index, _)| index)
                .unwrap();

            path.push(best_child_index);

            if let Some(m) = &current_node.children[best_child_index].m {
                board.make_move(m);
            }

            current_node = &mut current_node.children[best_child_index];
            self.nodes_visited += 1;
        }

        if board.get_result().is_end() {
            return path;
        }

        if !current_node.expanded {
            let legal_moves = board.get_total_legal_moves(None);

            if current_node.children.len() >= legal_moves.len() {
                current_node.expanded = true;
                return path;
            }

            let tried_moves: Vec<Move> = current_node.children.iter()
                .filter_map(|child| child.m.clone())
                .collect();

            for m in legal_moves {
                if !tried_moves.contains(&m) {
                    let child = Node::new(Some(m.clone()));

                    board.make_move(&m);
                    
                    current_node.children.push(child);
                    path.push(current_node.children.len() - 1);

                    self.nodes_visited += 1;
                    break;
                }
            }
        }
        
        path
    }

    fn simulate(&mut self, board: &mut Board) -> f64 {
        let turn = board.turn;
        let mut rng = rand::rng();
        let mut plies = 0;

        while !board.get_result().is_end() && plies < MCTS_MAX_PLIES && !self.is_stopping {
            let legal_moves = board.get_total_legal_moves(None);
            if legal_moves.is_empty() {
                break;
            }

            let mut weighted_moves = Vec::new();

            for m in &legal_moves {
                let mut weight = 1;
                if m.captured.is_some() {
                    let val = m.mvv_lva();
                    if val > 0.0 {
                        weight += 2;
                    }
                }

                for _ in 0..weight {
                    weighted_moves.push(m.clone());
                }
            }

            let random_move = weighted_moves.choose(&mut rng)
                .or_else(|| legal_moves.get(0))
                .expect("No moves");

            board.make_move(random_move);
            plies += 1;
        }

        match board.get_result() {
            ResultType::WhiteCheckmate => {
                if turn == PieceColor::White { 1.0 } else { 0.0 }
            },
            ResultType::BlackCheckmate => {
                if turn == PieceColor::Black { 1.0 } else { 0.0 }
            },
            ResultType::Draw | ResultType::Stalemate => 0.5,
            ResultType::None | ResultType::NotCached => {
                let eval = evaluate(board);
                let score = match turn {
                    PieceColor::White => eval.white - eval.black,
                    PieceColor::Black => eval.black - eval.white
                };

                0.5 + (score.tanh() / 2.0)
            }
        }
    }

    fn backpropagate(&mut self, root: &mut Node, path: &Vec<usize>, result: f64) {
        root.visits += 1;
        root.score += result;

        let mut current = root;
        for &index in path {
            current = &mut current.children[index];
            current.visits += 1;
            current.score += result;
        }
    }

    pub fn iterative_deepening(&mut self, board: &mut Board, time_chunks: u32, max_time_ms: u64) -> Option<Move> {
        let base_time = max_time_ms / time_chunks as u64;

        let mut best_move = None;
        let mut total_time_used = 0;

        for i in 1..=time_chunks {
            total_time_used += base_time;

            if self.is_stopping {
                break;
            }

            self.nodes_visited = 0;
            let m = self.search(board, base_time);

            best_move = Some(m);

            println!("MCTS iteration {}/{}: time used {}ms, total {}ms", 
                i, time_chunks, base_time, total_time_used);

            if total_time_used > max_time_ms * 9/10 {
                break;
            }
        }

        if self.is_stopping {
            self.reset_stop();
        }

        best_move
    }

    pub fn stop(&mut self) {
        self.is_stopping = true;
    }

    pub fn reset_stop(&mut self) {
        self.is_stopping = false;
    }
}

#[test]
fn test_mcts() {
    let mut board = Board::from_fen("2k2r2/1ppp4/pn5q/8/8/8/3B1PPP/1Q4K1 w - - 0 1");
    let mut mcts = Mcts::new();

    let best_move = mcts.iterative_deepening(&mut board, 20000, 10);
    println!("Best move: {:?}", best_move);
}