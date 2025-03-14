use std::cell::RefCell;
use std::rc::Rc;
use std::fmt;
use crate::piece::{Piece, PieceType, PieceColor, PieceRef};

#[derive(Debug, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize
}

impl Position {
    pub fn clamp(_n: isize) -> usize {
        if _n < 0 { 10 } else { _n as usize }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vector {
    pub x: isize,
    pub y: isize
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
    pub captured: Option<PieceRef>,
    pub promote_to: Option<PieceType>,
    pub piece_index: usize,
    pub piece_color: PieceColor,
    pub piece_type: PieceType,
    pub with: Option<PieceRef>
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let piece_char = match self.piece_type {
            PieceType::Pawn => "",
            PieceType::Knight => "N",
            PieceType::Bishop => "B",
            PieceType::Rook => "R",
            PieceType::Queen => "Q",
            PieceType::King => "K",
            _ => ""
        };

        let file_char = "abcdefgh".chars().nth(self.to.x).unwrap();

        write!(f, "{}{}{}", piece_char, file_char, 8 - self.to.y);

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Pin {
    pub position: Position,
    pub to: Position,
    pub color: PieceColor
}