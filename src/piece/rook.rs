use super::*;
use crate::board::ControlType;

#[derive(Clone)]
pub struct Rook {
    pub base: BasePiece
}

impl Rook {
    pub fn new(board: Board, color: PieceColor, pos: Position, index: usize) -> Self {
        Rook {
            base: BasePiece::new(board, PieceType::Rook, color, pos, index, true)
        }
    }
}

pub const ROOK_DIRECTIONS: [Vector; 4] = [Vector { x: -1, y: 0 }, Vector { x: 1, y: 0 }, Vector { x: 0, y: -1 }, Vector { x: 0, y: 1}];

impl Piece for Rook {
    fn clone_to(&self, board: Board) -> PieceRef {
        Rc::new(RefCell::new(Rook {
            base: self.base.clone_to(board)
        }))
    }

    fn get_legal_moves(&mut self) -> Vec<Move> {
        if self.base.legal_moves_cache.len() > 0 || !self.base.legal_moves { return self.base.legal_moves_cache.clone() };

        let file = self.base.pos.x;
        let rank = self.base.pos.y;

        let base = &self.base;
        let board = &self.base.board;
        
        let check_info = board.check.get(&base.color.clone());

        if board.is_pinned(rank, file) { return vec![] };
        if check_info.is_some_and(|c| c.double_checked) { return vec![] };

        let mut moves: Vec<Move> = vec![];

        for dir in ROOK_DIRECTIONS {
            for i in 1..9 {
                let t_file = Position::clamp(file as isize + dir.x * i);
                let t_rank = Position::clamp(rank as isize + dir.y * i);

                if !Board::in_bounds(t_rank, t_file) { break };

                let piece = board.get_piece_at(t_rank, t_file);

                if board.square_free(t_rank, t_file, base.color) {
                    moves.push(Move {
                        from: base.pos.clone(),
                        to: Position { x: t_file, y: t_rank },
                        move_type: [
                            if piece.is_some() {
                                MoveType::Capture
                            } else {
                                MoveType::Normal
                            }
                        ].to_vec(),
                        captured: piece,
                        promote_to: None,
                        piece_index: base.index,
                        piece_color: base.color,
                        piece_type: base.piece_type,
                        with: None
                    })
                }
            }
        }

        self.base.legal_moves_cache = moves.clone();
        if moves.len() == 0 { self.base.legal_moves = false; };

        moves
    }

    fn get_controlled_squares(&self) -> Vec<Control> {
        let file = self.base.pos.x;
        let rank = self.base.pos.y;

        let mut controlled: Vec<Control> = vec![];

        for dir in ROOK_DIRECTIONS {
            let mut obscured = false;
            for i in 1..9 {
                let t_file = Position::clamp(file as isize + dir.x * i);
                let t_rank = Position::clamp(rank as isize + dir.y * i);
    
                if !Board::in_bounds(t_rank, t_file) { continue };
    
                let piece = self.base.board.get_piece_at(t_rank, t_file);
    
                controlled.push(Control { 
                    pos: Position { x: t_file, y: t_rank }, 
                    control_type: if piece.as_ref().is_some_and(|p| p.borrow().get_base().color == self.base.color) {
                        ControlType::Defend
                    } else if piece.as_ref().is_some() {
                        ControlType::Attack
                    } else {
                        ControlType::Control
                    },
                    color: self.base.color, 
                    direction: Some(dir),
                    obscured
                });

                if piece.as_ref().is_some_and(|p| p.borrow().get_base().piece_type != PieceType::King) { break };
                if piece.as_ref().is_some() { obscured = true };
            }
        }

        controlled
    }

    fn get_pins(&self) -> Vec<Pin> {
        let file = self.base.pos.x;
        let rank = self.base.pos.y;

        let mut pins: Vec<Pin> = vec![];

        for dir in ROOK_DIRECTIONS {
            let mut enemy_piece: Option<PieceRef> = None;
            for i in 1..9 {
                let t_file = Position::clamp(file as isize + dir.x * i);
                let t_rank = Position::clamp(rank as isize + dir.y * i);

                if !Board::in_bounds(t_rank, t_file) { break };

                let piece = self.base.board.get_piece_at(t_rank, t_file);
                if piece.as_ref().is_some_and(|p| p.borrow().get_base().piece_type == PieceType::King) {
                    if piece.as_ref().unwrap().borrow().get_base().color == self.base.color { break };
                    if enemy_piece.as_ref().is_some() {
                        pins.push(Pin { 
                            position: enemy_piece.clone().unwrap().borrow().get_base().pos.clone(),
                            to: Position { x: t_file, y: t_rank },
                            color: self.base.color
                        })
                    } else {
                        enemy_piece = piece.clone();
                    }
                }
            }
        }

        pins
    }

    fn get_base(&self) -> &BasePiece {
        &self.base
    }

    fn get_base_mut(&mut self) -> &mut BasePiece {
        &mut self.base
    }
}