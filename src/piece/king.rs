use std::ops::Deref;

use crate::board::ControlType;

use super::*;

#[derive(Clone)]
pub struct King {
    pub base: BasePiece
}

impl King {
    pub fn new(board: Board, color: PieceColor, pos: Position, index: usize) -> Self {
        King {
            base: BasePiece::new(board, PieceType::King, color, pos, index, false)
        }
    }

    fn can_move_to(&self, rank: usize, file: usize, explicit: bool) -> bool {
        if !Board::in_bounds(rank, file) { return false };
        if self.base.board.get_control_at(rank, file, Some(self.base.color.opposite())).len() > 0 { return false };
        if explicit {
            self.base.board.is_empty(rank, file)
        } else {
            self.base.board.square_free(rank, file, self.base.color)
        }
    }

    fn can_move_multifile(&self, rank: usize, files: Vec<usize>) -> bool {
        let mut cond = true;
        for i in files {
            cond |= self.can_move_to(rank, i, true);
        }
        cond
    }
}

impl Piece for King {
    fn clone_to(&self, board: Board) -> PieceRef {
        Rc::new(RefCell::new(King {
            base: self.base.clone_to(board)
        }))
    }

    fn get_legal_moves(&mut self) -> Vec<Move> {
        if self.base.legal_moves_cache.len() > 0 || !self.base.legal_moves { return self.base.legal_moves_cache.clone() };

        let mut moves: Vec<Move> = vec![];
        let file = self.base.pos.x;
        let rank = self.base.pos.y;

        let base = &self.base;
        let board = &self.base.board;

        for i in 0..=2 {
            for j in 0..=2 {
                if i == 1 && j == 1 { continue };
                let t_rank = rank + i - 1;
                let t_file = file + j - 1;
                let piece = board.get_piece_at(t_rank, t_file);

                if self.can_move_to(rank, file, false) {
                    moves.push(Move {
                        from: base.pos.clone(),
                        to: Position { x: t_file, y: t_rank },
                        captured: piece.clone(),
                        move_type: [
                            match piece {
                                Some(_) => MoveType::Capture,
                                None => MoveType::Normal
                            }
                        ].to_vec(),
                        promote_to: None,
                        piece_index: base.index,
                        piece_color: base.color,
                        piece_type: base.piece_type,
                        with: None
                    })
                }
            }
        }

        if board.castling.can_castle_ks(base.color) && self.can_move_multifile(rank, [ file + 1, file + 2 ].to_vec()) {
            moves.push(Move {
                from: base.pos.clone(),
                to: Position { x: file, y: rank + 2 },
                captured: None,
                move_type: [ MoveType::Castling ].to_vec(),
                promote_to: None,
                piece_index: base.index,
                piece_color: base.color,
                piece_type: base.piece_type,
                with: board.get_piece_at(rank, file + 3)
            })
        }

        if board.castling.can_castle_qs(base.color) && self.can_move_multifile(rank, [ file - 1, file - 2, file - 3 ].to_vec()) {
            moves.push(Move {
                from: base.pos.clone(),
                to: Position { x: file, y: rank - 2 },
                captured: None,
                move_type: [ MoveType::Castling ].to_vec(),
                promote_to: None,
                piece_index: base.index,
                piece_color: base.color,
                piece_type: base.piece_type,
                with: board.get_piece_at(rank, file - 4)
            })
        }

        self.base.legal_moves_cache = moves.clone();
        if moves.len() == 0 { self.base.legal_moves = false; };

        moves
    }

    fn get_controlled_squares(&self) -> Vec<Control> {
        let file = self.base.pos.x;
        let rank = self.base.pos.y;

        let mut controlled: Vec<Control> = vec![];

        for i in -1..=1 {
            for j in -1..=1 {
                if i == 0 && j == 0 { continue };
                let t_file = Position::clamp(file as isize + i);
                let t_rank = Position::clamp(rank as isize + j);

                if !Board::in_bounds(t_rank, t_file) { continue };

                let piece = self.base.board.get_piece_at(t_rank, t_file);

                controlled.push(Control { 
                    pos: Position { x: t_file, y: t_rank }, 
                    control_type: if piece.as_ref().is_some_and(|p| p.borrow().deref().get_base().color == self.base.color) {
                        ControlType::Defend
                    } else if piece.is_some() {
                        ControlType::Attack
                    } else {
                        ControlType::Control
                    },
                    color: self.base.color, 
                    direction: None,
                    obscured: false
                })
            }
        }

        controlled
    }

    fn get_pins(&self) -> Vec<Pin> {
        vec![]
    }

    fn get_base(&self) -> &BasePiece {
        &self.base
    }

    fn get_base_mut(&mut self) -> &mut BasePiece {
        &mut self.base
    }
}