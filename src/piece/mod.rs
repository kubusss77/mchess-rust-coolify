use dead::Dead;

use crate::board::Castling;
use crate::moves::*;
use crate::board::Board;
use crate::board::Control;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

pub mod pawn;
pub mod knight;
pub mod bishop;
pub mod rook;
pub mod queen;
pub mod king;
pub mod dead;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
    Dead
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

pub trait Piece {
    fn get_base(&self) -> &BasePiece;
    fn get_base_mut(&mut self) -> &mut BasePiece;
    fn clone_to(&self, board: Board) -> PieceRef;
    fn get_legal_moves(&mut self) -> Vec<Move>;
    fn get_controlled_squares(&self) -> Vec<Control>;
    fn get_pins(&self) -> Vec<Pin>;
}

// pub trait PieceClone {
//     fn clone_box(&self) -> PieceRef;
// }

// impl<T> PieceClone for T
// where
//     T: 'static + Piece + Clone,
// {
//     fn clone_box(&self) -> PieceRef {
//         RefCell::new(self.clone())
//     }
// }

// impl Clone for PieceRef {
//     fn clone(&self) -> PieceRef {
//         self.clone_box()
//     }
// }

#[derive(Clone)]
pub struct BasePiece {
    pub piece_type: PieceType,
    pub color: PieceColor,
    pub board: Board,
    pub pos: Position,
    pub index: usize,
    pub legal_moves_cache: Vec<Move>,
    pub legal_moves: bool,
    _directional: bool,
}

impl BasePiece {
    pub fn new(board: Board, piece_type: PieceType, color: PieceColor, pos: Position, index: usize, _directional: bool) -> BasePiece {
        BasePiece {
            board,
            piece_type,
            color,
            pos,
            index,
            legal_moves_cache: vec![],
            legal_moves: true,
            _directional
        }
    }

    fn promote_to(&mut self, piece_type: PieceType) {

    }

    fn clone_to(&self, board: Board) -> BasePiece {
        let mut piece = self.clone();
        piece.board = board;
        piece
    }

    pub fn to_piece_index(&self) -> usize {
        return match self.piece_type {
            PieceType::Pawn => 0,
            PieceType::Knight => 1,
            PieceType::Bishop => 2,
            PieceType::Rook => 3,
            PieceType::Queen => 4,
            PieceType::King => 5,
            _ => unreachable!()
        } + if self.color == PieceColor::White { 0 } else { 6 }
    }

    pub fn move_(&mut self, move_: Move) {
        if move_.move_type.contains(&MoveType::Capture) && move_.captured.as_ref().is_some() {
            let captured = move_.captured.clone().unwrap();

            let base = captured.borrow().get_base();
            let pos = base.pos.clone();

            let dead_piece = Rc::new(RefCell::new(Dead::new(base.board, base.color, Position { x: 8, y: 8 }, base.index)));

            self.board.pieces.remove(base.index);
            self.board.pieces.insert(base.index, dead_piece);

            let captured_piece_index = base.to_piece_index();

            self.board.hash ^= self.board.hash_table[captured_piece_index * 64 + pos.y * 8 + pos.x];
        }
        
        let piece_index = self.to_piece_index();
        self.board.hash ^= self.board.hash_table[piece_index * 64 + move_.from.y * 8 + move_.from.x];
        self.board.hash ^= self.board.hash_table[piece_index * 64 + move_.to.y * 8 + move_.to.x];

        self.move_to(move_.to.x, move_.to.y, true);

        if move_.move_type.contains(&MoveType::Promotion) && move_.promote_to.as_ref().is_some() {
            self.promote_to(move_.promote_to.clone());
        }

        if self.piece_type == PieceType::King && move_.move_type.contains(&MoveType::Castling) && move_.with.as_ref().is_some() {
            let mut borrow = move_.with.unwrap().borrow_mut();
            let mut base = borrow.get_base_mut();
            base.move_(Move {
                from: base.pos.clone(),
                to: Position { x: if self.pos.x == 2 { 3 } else { 5 }, y: self.pos.y },
                move_type: [ MoveType::Castling ].to_vec(),
                captured: None,
                promote_to: None,
                piece_index: base.index,
                piece_color: base.color,
                piece_type: base.piece_type,
                with: None
            })
        }
    }

    pub fn move_to(&mut self, x: usize, y: usize, force: bool) {
        if self.index == -1 { return };
        
        let pos = self.pos;
        let current_board_pos = self.board.board[pos.y][pos.x];
        let current_piece = self.board.pieces.get(current_board_pos as usize).and_then(|p| Some(p.clone()));

        if let Some(piece_ref) = current_piece {
            let piece = piece_ref.borrow();
            if (!force && current_board_pos != -1) || piece.get_base().piece_type == PieceType::King { return false };
        }

        self.board.board[pos.y][pos.x] = -1;
        self.board.board[y][x] = self.index;

        self.pos = Position { x, y };

        self.check_control();

        for e in self.board.control_table[pos.y][pos.x] {
            self.board.pieces[e.index].borrow_mut().get_base_mut().check_control();
        }

        for e in self.board.control_table[y][x] {
            self.board.pieces[e.index].borrow_mut().get_base_mut().check_control();
        }
    }

    pub fn check_control(&mut self) {
        todo!()
    }
}

pub type PieceRef = Rc<RefCell<dyn Piece>>;