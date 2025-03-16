use crate::board::{Board, Control, ControlType};
use crate::moves::{Move, MoveType, Position, Vector};
use crate::piece::Piece;

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

pub fn get_legal_moves_knight(piece: &Piece, board: &Board) -> Vec<Move> {
    let file = piece.pos.x;
    let rank = piece.pos.y;
    
    let check_info = board.check.get(&piece.color);

    if board.is_pinned(rank, file) { return Vec::with_capacity(0) };
    if check_info.is_some_and(|c| c.double_checked) { return Vec::with_capacity(0) };

    let mut moves: Vec<Move> = Vec::with_capacity(8);

    for dir in KNIGHT_DIRECTIONS {
        let t_file = Position::clamp(file as isize + dir.x);
        let t_rank = Position::clamp(rank as isize + dir.y);

        let other = board.get_piece_at(t_rank, t_file);

        if board.square_free(t_rank, t_file, piece.color) {
            moves.push(Move {
                from: piece.pos,
                to: Position { x: t_file, y: t_rank },
                move_type: vec![
                    if other.is_some() {
                        MoveType::Capture
                    } else {
                        MoveType::Normal
                    }
                ],
                captured: other,
                promote_to: None,
                piece_index: piece.index,
                piece_color: piece.color,
                piece_type: piece.piece_type,
                with: None
            })
        }
    }

    moves
}

pub fn get_controlled_squares_knight(piece: &Piece, board: &Board) -> Vec<Control> {
    let file = piece.pos.x;
    let rank = piece.pos.y;

    let mut controlled: Vec<Control> = Vec::with_capacity(8);

    for dir in KNIGHT_DIRECTIONS {
        let t_file = Position::clamp(file as isize + dir.x);
        let t_rank = Position::clamp(rank as isize + dir.y);

        if !Board::in_bounds(t_rank, t_file) { continue };

        let other = board.get_piece_at(t_rank, t_file);

        controlled.push(Control { 
            pos: Position { x: t_file, y: t_rank }, 
            control_type: if other.as_ref().is_some_and(|p| p.color == piece.color) {
                ControlType::Defend
            } else if other.as_ref().is_some() {
                ControlType::Attack
            } else {
                ControlType::Control
            },
            color: piece.color, 
            direction: None,
            obscured: false
        })
    }

    controlled
}