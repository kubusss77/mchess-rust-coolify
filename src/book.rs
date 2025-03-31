use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use crate::board::Board;
use crate::moves::Move;

#[derive(Debug, Clone)]
pub struct OpeningBook {
    root: BookNode,
}

#[derive(Debug, Clone)]
struct BookNode {
    moves: HashMap<String, usize>,
    children: HashMap<String, BookNode>,
}

impl BookNode {
    fn new() -> Self {
        BookNode {
            moves: HashMap::new(),
            children: HashMap::new(),
        }
    }
}

impl OpeningBook {
    pub fn new() -> Self {
        OpeningBook {
            root: BookNode::new(),
        }
    }

    pub fn load_pgn_file<P: AsRef<Path>>(&mut self, file_path: P) -> io::Result<usize> {
        let file = File::open(file_path)?;
        let reader = io::BufReader::new(file);
        
        let mut in_game = false;
        let mut moves = Vec::new();
        let mut loaded_games = 0;
        
        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            
            if trimmed.is_empty() {
                continue;
            }
            
            if trimmed.starts_with('[') {
                if in_game && !moves.is_empty() {
                    self.add_game(&moves);
                    moves.clear();
                    loaded_games += 1;
                }
                in_game = false;
                continue;
            }
            
            in_game = true;
            
            for token in trimmed.split_whitespace() {
                if token.contains('.') && !token.starts_with('.') {
                    let parts: Vec<&str> = token.split('.').collect();
                    if parts.len() > 1 && !parts[1].is_empty() {
                        let m = parts[1];
                        self.process_token(m, &mut moves)?;
                    }
                } else {
                    self.process_token(token, &mut moves)?;
                }
            }
        }
        
        if !moves.is_empty() {
            self.add_game(&moves);
            loaded_games += 1;
        }
        
        Ok(loaded_games)
    }
    
    fn process_token(&mut self, token: &str, moves: &mut Vec<String>) -> io::Result<()> {
        if token.parse::<u32>().is_ok() {
            return Ok(());
        }
        
        if token == "1-0" || token == "0-1" || token == "1/2-1/2" || token == "*" {
            return Ok(());
        }
        
        let m = token.trim_end_matches(&['+', '#', '!', '?'][..])
            .split(';').next().unwrap()
            .split('{').next().unwrap()
            .to_string();
        
        if m.is_empty() || m == "0-0" || m == "0-0-0" {
            return Ok(());
        }

        moves.push(m);
        
        Ok(())
    }

    fn add_game(&mut self, moves: &[String]) {
        if moves.is_empty() {
            return;
        }
    
        let mut current = &mut self.root;
    
        for m in moves {
            *current.moves.entry(m.clone()).or_insert(0) += 1;
            
            current = current.children
                .entry(m.clone())
                .or_insert_with(BookNode::new);
        }
    }
    
    pub fn get_best_move(&self, moves: &[String]) -> Option<String> {
        let mut current = &self.root;
        
        for mv in moves {
            match current.children.get(mv) {
                Some(child) => current = child,
                None => return None,
            }
        }
        
        current.moves.iter()
            .max_by_key(|&(_, count)| count)
            .map(|(mv, _)| mv.clone())
    }

    pub fn to_move(&self, san: &str, board: &mut Board) -> Option<Move> {
        let mut found = None;
        for m in board.get_total_legal_moves(None) {
            if m.to_san(board) == san {
                found = Some(m);
            }
        }
        
        found
    }

    pub fn print_statistics(&self) {
        let mut first_moves: Vec<_> = self.root.moves.iter().collect();
        
        first_moves.sort_by(|&(_, count1), &(_, count2)| count2.cmp(count1));
        
        if first_moves.is_empty() {
            println!("info string book: no moves in book");
            return;
        }
        
        println!("info string book: first branches");
        
        for (mv, count) in first_moves {
            match self.root.children.get(mv) {
                Some(node) => {
                    let total_branch_moves = self.count_all_branch_moves(node);
                    
                    println!("info string book: - {} - played {} times, {} total sub mov stored", mv, count, total_branch_moves);
                },
                None => {
                    println!("info string book: - {} - played {} times", mv, count);
                }
            }
        }
        
        let total_positions = self.count_total_positions();
        let total_moves = self.count_total_moves();
        
        println!("\ninfo string book: total pos: {}", total_positions);
        println!("info string book: total mov: {}", total_moves);
    }

    fn count_all_branch_moves(&self, node: &BookNode) -> usize {
        let mut count = node.moves.len();
        
        for (_, child) in &node.children {
            count += self.count_all_branch_moves(child);
        }
        
        count
    }

    fn count_total_positions(&self) -> usize {
        let mut count = 1;
        
        for (_, child) in &self.root.children {
            count += self.count_positions_recursive(child);
        }
        
        count
    }

    fn count_positions_recursive(&self, node: &BookNode) -> usize {
        let mut count = 1;
        
        for (_, child) in &node.children {
            count += self.count_positions_recursive(child);
        }
        
        count
    }

    fn count_total_moves(&self) -> usize {
        let mut count = self.root.moves.len();
        
        for (_, child) in &self.root.children {
            count += self.count_all_branch_moves(child);
        }
        
        count
    }
}