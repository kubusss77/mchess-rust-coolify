use std::fmt;
use crate::board::Board;
use crate::evaluation::evaluate_position;
use crate::r#const::{MVV_LVA_VALUES, PIECE_VALUES};
use crate::piece::{PieceType, PieceColor, Piece};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

#[derive(Clone, Copy, Hash)]
pub struct Position {
    pub x: usize,
    pub y: usize
}

impl Position {
    pub fn from(x: isize, y: isize) -> Self {
        Position {
            x: Position::clamp(x),
            y: Position::clamp(y)
        }
    }

    pub fn clamp(_n: isize) -> usize {
        if _n < 0 { 10 } else { _n as usize }
    }

    pub fn to_vector(&self) -> Vector {
        Vector {
            x: self.x as isize,
            y: self.y as isize
        }
    }

    pub fn to_bitboard(&self) -> u64 {
        1u64 << (self.x + self.y * 8)
    }

    pub fn from_bitboard(square: u64) -> Self {
        let index = square.trailing_zeros() as usize;
        Position { x: index % 8, y: index / 8 }
    }

    pub fn is_bigger_than(&self, base: Position, dir: Vector) -> bool {
        let dx = self.x as isize - base.x as isize;
        let dy = self.y as isize - base.y as isize;

        if dx == 0 && dy == 0 {
            return false;
        }

        if dir.x == 0 {
            if dx != 0 {
                return false;
            }
            return dy * dir.y > 0;
        } else if dir.y == 0 {
            if dy != 0 {
                return false;
            }
            return dx * dir.x > 0;
        } else {
            if dx.abs() != dy.abs() {
                return false;
            }
            return dx * dir.x > 0 && dy * dir.y > 0;
        }
    }

    pub fn shift(&self, vec: Vector) -> Position {
        Position::from(self.x as isize + vec.x, self.y as isize + vec.y)
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file_char = "abcdefgh".chars().nth(self.x).unwrap();

        write!(f, "{}{}", file_char, 8 - self.y)
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vector {
    pub x: isize,
    pub y: isize
}

impl Vector {
    pub fn in_direction(&self, pos1: Position, pos2: Position) -> bool {
        let vec1 = pos1.to_vector();
        let vec2 = pos2.to_vector();
        vec1.x * self.x >= vec1.x * self.x && vec2.y * self.y >= vec2.y * self.y
    }

    pub fn between(pos1: Position, pos2: Position) -> Vector {
        let vec1 = pos1.to_vector();
        let vec2 = pos2.to_vector();
        let x_diff = vec2.x - vec1.x;
        let y_diff = vec2.y - vec1.y;

        Vector {
            x: x_diff.signum(),
            y: y_diff.signum()
        }
    }
 
    pub fn is_parallel_to(&self, other: Vector) -> bool {
        self.x * other.y == self.y * other.x
    }

    pub fn inv(&self) -> Vector {
        Vector {
            x: -self.x,
            y: -self.y
        }
    }
}

impl PartialEq for Vector {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MoveType {
    Normal,
    Capture,
    Castling,
    Promotion,
    Check,
    EnPassant
}

#[derive(Clone)]
pub struct Move {
    pub from: Position,
    pub to: Position,
    pub move_type: Vec<MoveType>,
    pub captured: Option<Piece>,
    pub promote_to: Option<PieceType>,
    pub piece_index: usize,
    pub piece_color: PieceColor,
    pub piece_type: PieceType,
    pub with: Option<Piece>
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let promotion_char = if let Some(piece_type) = self.promote_to {
            match piece_type {
                PieceType::Knight => "n",
                PieceType::Bishop => "b",
                PieceType::Rook => "r",
                PieceType::Queen => "q",
                _ => unreachable!()
            }
        } else {
            ""
        };

        write!(f, "{:?}{:?}{}", self.from, self.to, promotion_char)
    }
}

impl Move {
    pub fn hash(&self) -> usize {
        let mut hasher = DefaultHasher::new();
        self.from.hash(&mut hasher);
        self.to.hash(&mut hasher);
        self.promote_to.hash(&mut hasher);
        self.piece_index.hash(&mut hasher);
        self.piece_color.hash(&mut hasher);
        self.piece_type.hash(&mut hasher);
        hasher.finish() as usize
    }

    pub fn mvv_lva(&self) -> f64 {
        if !self.move_type.contains(&MoveType::Capture) || self.captured.is_none() {
            return 0.0;
        }
        let victim = self.captured.as_ref().unwrap().piece_type.index();
        let aggressor = self.piece_type.index();
        
        let ordering_value = MVV_LVA_VALUES[victim][aggressor] as f64;
        
        let victim_value = PIECE_VALUES[victim];
        let aggressor_value = PIECE_VALUES[aggressor];
        
        if aggressor_value > victim_value {
            let trade_penalty = (aggressor_value - victim_value) * 2.0;
            return ordering_value - trade_penalty;
        }

        ordering_value
    }

    pub fn ps_table(&self, board: &Board) -> f64 {
        let x = self.to.x;
        let y = self.to.y;

        let y_index = if self.piece_color == PieceColor::White { y } else { 7 - y };

        evaluate_position(board, self.piece_type, x, y_index)
    }

    pub fn to_san(&self, board: &Board) -> String {
        if self.move_type.contains(&MoveType::Castling) {
            if self.to.x == 6 {
                return "O-O".to_string();
            } else {
                return "O-O-O".to_string();
            }
        }

        let mut san = String::new();

        if self.piece_type != PieceType::Pawn {
            san.push(match self.piece_type {
                PieceType::King => 'K',
                PieceType::Queen => 'Q',
                PieceType::Rook => 'R',
                PieceType::Bishop => 'B',
                PieceType::Knight => 'N',
                _ => unreachable!()
            });
            
            let mut same_pieces = Vec::new();
            
            for (i, piece) in board.pieces.iter() {
                if piece.piece_type == self.piece_type && 
                   piece.color == self.piece_color && 
                   *i != self.piece_index {
                    
                    let moves = board.get_legal_moves(*i);
                    if moves.iter().any(|m| m.to == self.to) {
                        same_pieces.push(i);
                    }
                }
            }
        
            if !same_pieces.is_empty() {
                let from_file = self.from.x;
                let from_rank = self.from.y;
                
                let need_file = same_pieces.iter().any(|&index| {
                    board.pieces[&index].pos.x == from_file
                });
                
                let need_rank = same_pieces.iter().any(|&index| {
                    board.pieces[&index].pos.y == from_rank
                });
                
                if !need_rank {
                    san.push("abcdefgh".chars().nth(from_file).unwrap());
                } else if !need_file {
                    san.push(char::from_digit(8 - from_rank as u32, 10).unwrap());
                } else {
                    san.push("abcdefgh".chars().nth(from_file).unwrap());
                    san.push(char::from_digit(8 - from_rank as u32, 10).unwrap());
                }
            }
        }

        if self.move_type.contains(&MoveType::Capture) {
            if self.piece_type == PieceType::Pawn {
                san.push("abcdefgh".chars().nth(self.from.x).unwrap());
            }
            san.push('x');
        }

        san.push("abcdefgh".chars().nth(self.to.x).unwrap());
        san.push(char::from_digit(8 - self.to.y as u32, 10).unwrap());

        if self.move_type.contains(&MoveType::Promotion) && self.promote_to.is_some() {
            san.push('=');
            san.push(match self.promote_to.unwrap() {
                PieceType::Queen => 'Q',
                PieceType::Rook => 'R',
                PieceType::Bishop => 'B',
                PieceType::Knight => 'N',
                _ => unreachable!()
            });
        }

        san
    }
}

impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from &&
        self.to == other.to
    }
}

impl Eq for Move {}

#[derive(Debug, Clone)]
pub struct Pin {
    pub position: Position,
    pub to: Position,
    pub color: PieceColor,
    pub dir: Vector,
    pub is_phantom: bool
}