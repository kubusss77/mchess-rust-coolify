use std::fmt;
use crate::piece::{PieceType, PieceColor, Piece};

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: usize,
    pub y: usize
}

impl Position {
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

#[derive(Debug, Clone)]
pub struct Pin {
    pub position: Position,
    pub to: Position,
    pub color: PieceColor
}