use super::*;

#[derive(Clone)]
pub struct Dead {
    pub base: BasePiece
}

impl Dead {
    pub fn new(board: Board, color: PieceColor, pos: Position, index: usize) -> Self {
        Dead {
            base: BasePiece::new(board, PieceType::Dead, color, pos, index, false)
        }
    }
}

impl Piece for Dead {
    fn get_base(&self) -> &BasePiece {
        &self.base
    }

    fn get_base_mut(&mut self) -> &mut BasePiece {
        &mut self.base
    }

    fn clone_to(&self, board: Board) -> PieceRef {
        Rc::new(RefCell::new(Dead {
            base: self.base.clone_to(board)
        }))
    }

    fn get_legal_moves(&mut self) -> Vec<Move> {
        vec![]
    }

    fn get_controlled_squares(&self) -> Vec<Control> {
        vec![]
    }

    fn get_pins(&self) -> Vec<Pin> {
        vec![]
    }
}