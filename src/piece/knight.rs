use super::*;
use crate::board::ControlType;

#[derive(Clone)]
pub struct Knight {
    pub base: BasePiece
}

impl Knight {
    pub fn new(board: Board, color: PieceColor, pos: Position, index: usize) -> Self {
        Knight {
            base: BasePiece::new(board, PieceType::Knight, color, pos, index, false)
        }
    }
}

const KNIGHT_DIRECTIONS: [Vector; 8] = [
    Vector { x: 2, y: -1 },
    Vector { x: 2, y: 1 },
    Vector { x: 1, y: 2 },
    Vector { x: -1, y: 2 },
    Vector { x: -2, y: 1 },
    Vector { x: -2, y: -1 },
    Vector { x: -1, y: -2 },
    Vector { x: 1, y: -2 }
];

impl Piece for Knight {
    fn clone_to(&self, board: Board) -> PieceRef {
        Rc::new(RefCell::new(Knight {
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

        for dir in KNIGHT_DIRECTIONS {
            let t_file = Position::clamp(file as isize + dir.x);
            let t_rank = Position::clamp(rank as isize + dir.y);

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

        self.base.legal_moves_cache = moves.clone();
        if moves.len() == 0 { self.base.legal_moves = false; };

        moves
    }

    fn get_controlled_squares(&self) -> Vec<Control> {
        let file = self.base.pos.x;
        let rank = self.base.pos.y;

        let mut controlled: Vec<Control> = vec![];

        for dir in KNIGHT_DIRECTIONS {
            let t_file = Position::clamp(file as isize + dir.x);
            let t_rank = Position::clamp(rank as isize + dir.y);

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