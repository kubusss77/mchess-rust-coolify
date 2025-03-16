use std::fmt;
use crate::piece::{PieceType, PieceColor, Piece};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

#[derive(Debug, Clone, Copy, Hash)]
pub struct Position {
    pub x: usize,
    pub y: usize
}

impl Position {
    pub fn from(x: isize, y: isize) -> Position {
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
    pub fn in_direction(&self, _pos1: Position, _pos2: Position) -> bool {
        let pos1 = _pos1.to_vector();
        let pos2 = _pos2.to_vector();
        pos2.x * self.x >= pos1.x * self.x && pos2.y * self.y >= pos1.y * self.y
    }
}

impl PartialEq for Vector {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

#[derive(Debug, Clone, PartialEq)]
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

impl Hash for Move {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.from.hash(state);
        self.to.hash(state);
        self.move_type.len().hash(state);

        if let Some(captured) = &self.captured {
            captured.piece_type.hash(state)
        }
        if let Some(promote_to) = &self.promote_to {
            promote_to.hash(state)
        }
        if let Some(with) = &self.with {
            with.piece_type.hash(state)
        }

        self.piece_index.hash(state);
        self.piece_color.hash(state);
        self.piece_type.hash(state);
    }
}

impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from &&
        self.to == other.from &&
        self.move_type.len() == other.move_type.len() &&
        self.captured.is_some() == other.captured.is_some() &&
        self.promote_to.is_some() == other.promote_to.is_some() &&
        self.piece_index == other.piece_index &&
        self.piece_color == other.piece_color &&
        self.piece_type == other.piece_type &&
        self.with.is_some() == other.with.is_some()
    }
}

impl Eq for Move {}

#[derive(Debug, Clone)]
pub struct Pin {
    pub position: Position,
    pub to: Position,
    pub color: PieceColor
}