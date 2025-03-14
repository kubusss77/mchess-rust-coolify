use super::*;
use crate::board::ControlType;

#[derive(Clone)]
pub struct Pawn {
    pub base: BasePiece
}

impl Pawn {
    pub fn new(board: Board, color: PieceColor, pos: Position, index: usize) -> Self {
        Pawn {
            base: BasePiece::new(board, PieceType::Pawn, color, pos, index, false)
        }
    }
}

impl Piece for Pawn {
    fn clone_to(&self, board: Board) -> PieceRef {
        Rc::new(RefCell::new(Pawn {
            base: self.base.clone_to(board)
        }))
    }

    fn get_legal_moves(&mut self) -> Vec<Move> {
        if self.base.legal_moves_cache.len() > 0 || !self.base.legal_moves { return self.base.legal_moves_cache.clone() };

        let file = self.base.pos.x;
        let rank = self.base.pos.y;

        let base = &self.base;
        let board = &self.base.board;

        let dir: isize = if base.color == PieceColor::White { -1 } else { 1 };

        let check_info = board.check.get(&base.color.clone());

        if board.is_pinned(rank, file) { return vec![] };
        if check_info.is_some_and(|c| c.double_checked) { return vec![] };

        let promotion_rank = if base.color == PieceColor::White { 7 } else { 0 };
        let initial_rank = if base.color == PieceColor::White { 6 } else { 2 };
        
        let advanced_rank = Position::clamp(rank as isize + dir);

        let mut moves: Vec<Move> = vec![];

        if board.is_empty(advanced_rank, file) {
            if advanced_rank == promotion_rank {
                for piece_type in [ PieceType::Knight, PieceType::Bishop, PieceType::Rook, PieceType::Queen ] {
                    moves.push(Move {
                        from: base.pos.clone(),
                        to: Position { x: file, y: advanced_rank },
                        move_type: [ MoveType::Promotion ].to_vec(),
                        captured: None,
                        promote_to: Some(piece_type),
                        piece_index: base.index,
                        piece_color: base.color,
                        piece_type: base.piece_type,
                        with: None
                    });
                }
            } else {
                moves.push(Move {
                    from: base.pos.clone(),
                    to: Position { x: file, y: advanced_rank },
                    move_type: [ MoveType::Normal ].to_vec(),
                    captured: None,
                    promote_to: None,
                    piece_index: base.index,
                    piece_color: base.color,
                    piece_type: base.piece_type,
                    with: None
                });
            }
        }

        if rank == initial_rank && board.is_empty(advanced_rank, file) && board.is_empty(Position::clamp(rank as isize + dir * 2), file) {
            moves.push(Move {
                from: base.pos.clone(),
                to: Position { x: file, y: Position::clamp(rank as isize + dir * 2) },
                move_type: [ MoveType::Normal ].to_vec(),
                captured: None,
                promote_to: None,
                piece_index: base.index,
                piece_color: base.color,
                piece_type: base.piece_type,
                with: None
            });
        }

        for square in [-1, 1] {
            let advanced_file = Position::clamp(file as isize + square);
            if !Board::in_bounds(advanced_rank, advanced_file) { continue };
            let piece_ = board.get_piece_at(advanced_rank, advanced_file);
            if piece_.clone().is_some_and(|p| p.borrow().get_base().color != base.color) {
                if advanced_rank == promotion_rank {
                    for piece_type in [ PieceType::Knight, PieceType::Bishop, PieceType::Rook, PieceType::Queen ] {
                        moves.push(Move {
                            from: base.pos.clone(),
                            to: Position { x: advanced_file, y: advanced_rank },
                            move_type: [ MoveType::Promotion, MoveType::Capture ].to_vec(),
                            captured: piece_.clone(),
                            promote_to: Some(piece_type),
                            piece_index: base.index,
                            piece_color: base.color,
                            piece_type: base.piece_type,
                            with: None
                        });
                    }
                } else {
                    moves.push(Move {
                        from: base.pos.clone(),
                        to: Position { x: advanced_file, y: advanced_rank },
                        move_type: [ MoveType::Capture ].to_vec(),
                        captured: piece_.clone(),
                        promote_to: None,
                        piece_index: base.index,
                        piece_color: base.color,
                        piece_type: base.piece_type,
                        with: None
                    });
                }
            }
            
            if board.target_square.is_some() {
                let t_rank = board.target_square.clone().unwrap().x;
                let t_file = board.target_square.clone().unwrap().y;

                if advanced_rank == t_rank && advanced_file == t_file {
                    moves.push(Move {
                        from: base.pos.clone(),
                        to: Position { x: advanced_file, y: advanced_rank },
                        move_type: [ MoveType::Capture ].to_vec(),
                        captured: board.get_piece_at(rank, advanced_file),
                        promote_to: None,
                        piece_index: base.index,
                        piece_color: base.color,
                        piece_type: base.piece_type,
                        with: None
                    });
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

        let dir = if self.base.color == PieceColor::White { -1 } else { 1 };

        let mut controlled: Vec<Control> = vec![];

        for square in [-1, 1] {
            let t_file = Position::clamp(file as isize + square);
            let t_rank = Position::clamp(rank as isize + dir);

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
                direction: None,
                obscured: false
            })
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