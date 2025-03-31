use crate::moves::{Move, Position};


#[derive(Debug, Clone, PartialEq, Copy, Hash)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}

impl PieceType {
    pub fn to_value(&self) -> usize {
        match self {
            PieceType::Pawn => 1,
            PieceType::Bishop => 3,
            PieceType::Knight => 3,
            PieceType::Rook => 5,
            PieceType::Queen => 9,
            PieceType::King => 100
        }
    }
    
    pub fn is_directional(&self) -> bool {
        matches!(self, PieceType::Bishop | PieceType::Rook | PieceType::Queen)
    }
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
    pub legal_moves: bool
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

    pub fn piece_index(piece_type: PieceType, color: PieceColor) -> usize {
        return match piece_type {
            PieceType::Pawn => 0,
            PieceType::Knight => 1,
            PieceType::Bishop => 2,
            PieceType::Rook => 3,
            PieceType::Queen => 4,
            PieceType::King => 5
        } + if color == PieceColor::White { 0 } else { 6 }
    }

    pub fn to_partial(&self) -> PartialPiece {
        PartialPiece { piece_type: self.piece_type, pos: self.pos, color: self.color }
    }

    pub fn get_base(&self) -> BasePiece {
        (self.piece_type, self.color)
    }
}

#[derive(Debug, Clone)]
pub struct PartialPiece {
    pub piece_type: PieceType,
    pub pos: Position,
    pub color: PieceColor
}

pub type BasePiece = (PieceType, PieceColor);