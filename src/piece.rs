use crate::moves::{Move, Position};


#[derive(Debug, Clone, PartialEq, Copy)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum PieceColor {
    White,
    Black
}

impl PieceColor {
    pub fn opposite(&self) -> PieceColor {
        match self {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White
        }
    }
}

#[derive(Debug, Clone)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: PieceColor,
    pub pos: Position,
    pub index: usize,
    pub legal_moves_cache: Vec<Move>,
    pub legal_moves: bool,
    pub has_moved: bool,
    pub _directional: bool,
}

impl Piece {
    pub fn to_piece_index(&self) -> usize {
        return match self.piece_type {
            PieceType::Pawn => 0,
            PieceType::Knight => 1,
            PieceType::Bishop => 2,
            PieceType::Rook => 3,
            PieceType::Queen => 4,
            PieceType::King => 5
        } + if self.color == PieceColor::White { 0 } else { 6 }
    }
}