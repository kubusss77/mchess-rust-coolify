use std::fmt;
use crate::board::Board;
use crate::evaluation::evaluate_position;
use crate::r#const::MVV_LVA_VALUE;
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
        let x_diff = vec1.x - vec2.x;
        let y_diff = vec1.y - vec2.y;

        Vector {
            x: x_diff.signum(),
            y: y_diff.signum()
        }
    }
 
    pub fn is_parallel_to(&self, other: Vector) -> bool {
        self.x * other.y == self.y * other.x
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
    Check
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
        let piece_char = match self.piece_type {
            PieceType::Pawn => "",
            PieceType::Knight => "N",
            PieceType::Bishop => "B",
            PieceType::Rook => "R",
            PieceType::Queen => "Q",
            PieceType::King => "K"
        };

        let file_char = "abcdefgh".chars().nth(self.to.x).unwrap();

        write!(f, "{}{}{}", piece_char, file_char, 8 - self.to.y)
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

        let victim = self.captured.as_ref().unwrap().piece_type;
        let aggressor = self.piece_type;

        let piece_value = |p: PieceType| -> f64 {
            match p {
                PieceType::Pawn => 100.0,
                PieceType::Knight => 300.0,
                PieceType::Bishop => 300.0,
                PieceType::Rook => 500.0,
                PieceType::Queen => 900.0,
                PieceType::King => 10000.0
            }
        };

        (piece_value(victim) - piece_value(aggressor)/10.0) * MVV_LVA_VALUE
    }

    pub fn ps_table(&self, board: &Board) -> f64 {
        let x = self.to.x;
        let y = self.to.y;

        let y_index = if self.piece_color == PieceColor::White { y } else { 7 - y };

        evaluate_position(board, self.piece_type, x, y_index)
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
    pub dir: Vector
}