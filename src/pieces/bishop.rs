use crate::board::{Board, Control, ControlType};
use crate::moves::{Move, MoveType, Pin, Position, Vector};
use crate::piece::{PartialPiece, Piece, PieceType};

pub const BISHOP_DIRECTIONS: [Vector; 4] = [Vector { x: -1, y: -1 }, Vector { x: -1, y: 1 }, Vector { x: 1, y: -1 }, Vector { x: 1, y: 1}];

pub fn get_legal_moves_bishop(piece: &Piece, board: &Board) -> Vec<Move> {
    let file = piece.pos.x;
    let rank = piece.pos.y;
    
    let check_info = board.check.get(&piece.color.clone());

    let pin_dir = board.is_pinned(rank, file);
    if check_info.is_some_and(|c| c.double_checked) { return Vec::with_capacity(0) };

    let mut moves: Vec<Move> = Vec::with_capacity(13);

    for &dir in &BISHOP_DIRECTIONS {
        if let Some(pin) = pin_dir {
            if pin.x != 0 && dir.y != 0 { continue; }
            if pin.y != 0 && dir.x != 0 { continue; }
        }
        for i in 1..9 {
            let t_file = Position::clamp(file as isize + dir.x * i);
            let t_rank = Position::clamp(rank as isize + dir.y * i);

            if !Board::in_bounds(t_rank, t_file) { break };

            let other = board.get_piece_at(t_rank, t_file);

            let flag = other.as_ref().is_some();
            
            if board.square_free(t_rank, t_file, piece.color) {
                moves.push(Move {
                    from: piece.pos,
                    to: Position { x: t_file, y: t_rank },
                    move_type: vec![
                        match &other {
                            Some(_) => MoveType::Capture,
                            None => MoveType::Normal
                        }; 1
                    ],
                    captured: other,
                    promote_to: None,
                    piece_index: piece.index,
                    piece_color: piece.color,
                    piece_type: piece.piece_type,
                    with: None
                })
            }

            if flag { break };
        }
    }

    moves
}

pub fn get_controlled_squares_bishop(piece: &PartialPiece, board: &Board) -> Vec<Control> {
    let file = piece.pos.x;
    let rank = piece.pos.y;

    let mut controlled: Vec<Control> = Vec::with_capacity(13);

    for &dir in &BISHOP_DIRECTIONS {
        let mut obscured = false;

        for i in 1..8 {
            let t_file = Position::clamp(file as isize + dir.x * i);
            let t_rank = Position::clamp(rank as isize + dir.y * i);

            if !Board::in_bounds(t_rank, t_file) { continue };

            let other = board.get_piece_at(t_rank, t_file);

            let control_type = match &other {
                Some(p) if p.color == piece.color => ControlType::Defend,
                Some(_) => ControlType::Attack,
                None => ControlType::Control
            };

            controlled.push(Control { 
                pos: Position { x: t_file, y: t_rank }, 
                control_type,
                color: piece.color, 
                direction: Some(dir),
                obscured
            });

            if let Some(p) = &other {
                if p.piece_type != PieceType::King {
                    break;
                }
                obscured = true;
            }
        }
    }

    controlled
}

pub fn get_pins_bishop(piece: &Piece, board: &Board) -> Vec<Pin> {
    let file = piece.pos.x;
    let rank = piece.pos.y;

    let mut pins: Vec<Pin> = Vec::with_capacity(4);

    for dir in BISHOP_DIRECTIONS {
        let mut enemy_piece: Option<Piece> = None;
        for i in 1..9 {
            let t_file = Position::clamp(file as isize + dir.x * i);
            let t_rank = Position::clamp(rank as isize + dir.y * i);

            if !Board::in_bounds(t_rank, t_file) { break };

            let other = board.get_piece_at(t_rank, t_file);
            if other.as_ref().is_some_and(|p| p.piece_type == PieceType::King) {
                if other.as_ref().unwrap().color == piece.color { break };
                if enemy_piece.is_some() {
                    pins.push(Pin { 
                        position: enemy_piece.clone().unwrap().pos,
                        to: Position { x: t_file, y: t_rank },
                        color: piece.color,
                        dir
                    })
                } else {
                    enemy_piece = other.clone();
                }
            }
        }
    }

    pins
}