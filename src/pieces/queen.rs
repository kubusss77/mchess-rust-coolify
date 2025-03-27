use crate::board::{Board, Control, ControlType};
use crate::moves::{Move, MoveType, Pin, Position, Vector};
use crate::piece::{PartialPiece, Piece, PieceColor, PieceType};

use super::bishop::generate_bishop_rays;
use super::rook::generate_rook_rays;

const QUEEN_DIRECTIONS: [Vector; 8] = [
    Vector { x: -1, y: -1 },
    Vector { x: -1, y: 1 },
    Vector { x: 1, y: -1 },
    Vector { x: 1, y: 1},
    Vector { x: -1, y: 0 },
    Vector { x: 1, y: 0 },
    Vector { x: 0, y: -1 },
    Vector { x: 0, y: 1}
];

fn generate_queen_rays(pos: u64, occupied: u64, enemy_king: u64, let_through: bool) -> (u64, u64) {
    let (b_attacks, b_obscured) = generate_bishop_rays(pos, occupied, enemy_king, let_through);
    let (r_attacks, r_obscured) = generate_rook_rays(pos, occupied, enemy_king, let_through);

    (b_attacks | r_attacks, b_obscured | r_obscured)
}

pub fn get_legal_moves_queen_bitboard(piece: &Piece, board: &Board) -> Vec<Move> {
    let pos = piece.pos.to_bitboard();
    let mut moves = Vec::with_capacity(27);

    let pin_dir = board.is_pinned(piece.pos.y, piece.pos.x);
    let check_info = board.check.get(&piece.color);

    let mut valid_squares = !0u64;
    if let Some(check_info) = check_info {
        if check_info.double_checked != 0u64 {
            return moves;
        }
        if check_info.block_mask != 0u64 { valid_squares = check_info.block_mask; }
    }

    let (attacks, _) = generate_queen_rays(pos, board.all_pieces, 0u64, false);

    let enemy = if piece.color == PieceColor::White {
        board.black_pieces
    } else {
        board.white_pieces
    };

    let valid_moves = attacks & (board.empty_squares | enemy) & valid_squares;

    let mut rem = valid_moves;
    while rem != 0 {
        let index = rem.trailing_zeros() as usize;
        let square = 1u64 << index;
        let to_pos = Position::from_bitboard(square);

        if let Some(pin) = pin_dir {
            let x_diff = (to_pos.x as isize - piece.pos.x as isize).signum();
            let y_diff = (to_pos.y as isize - piece.pos.y as isize).signum();

            let vec = Vector { x: x_diff, y: y_diff };

            if !vec.is_parallel_to(pin) {
                rem &= rem - 1;
                continue;
            }
        }

        let is_capture = square & enemy != 0;
        let captured = if is_capture { board.get_piece_at(to_pos.y, to_pos.x) } else { None };
        
        moves.push(Move {
            from: piece.pos,
            to: to_pos,
            move_type: vec![if is_capture { MoveType::Capture } else { MoveType::Normal }; 1],
            captured,
            promote_to: None,
            piece_index: piece.index,
            piece_color: piece.color,
            piece_type: piece.piece_type,
            with: None
        });

        rem &= rem - 1;
    }

    moves
}

pub fn get_legal_moves_queen(piece: &Piece, board: &Board) -> Vec<Move> {
    let file = piece.pos.x;
    let rank = piece.pos.y;
    
    let check_info = board.check.get(&piece.color.clone());

    let pin_dir = board.is_pinned(rank, file);
    if check_info.is_some_and(|c| c.double_checked != 0u64) { return Vec::with_capacity(0) };

    let mut moves: Vec<Move> = Vec::with_capacity(27);

    for &dir in &QUEEN_DIRECTIONS {
        if let Some(pin) = pin_dir {
            if !dir.is_parallel_to(pin) { continue; }
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

pub fn get_controlled_squares_queen_bitboard(piece: &PartialPiece, board: &Board) -> Vec<Control> {
    let pos = piece.pos.to_bitboard();
    let mut controlled = Vec::with_capacity(27);

    let (attacks, obscured) = generate_queen_rays(pos, board.all_pieces, if piece.color == PieceColor::White { board.black_king } else { board.white_king }, true);

    let friendly = if piece.color == PieceColor::White {
        board.white_pieces
    } else {
        board.black_pieces
    };

    let enemy = if piece.color == PieceColor::White {
        board.black_pieces
    } else {
        board.white_pieces
    };

    let mut rem = attacks;
    while rem != 0 {
        let index = rem.trailing_zeros() as usize;
        let square = 1u64 << index;
        let to_pos = Position::from_bitboard(square);

        let control_type = if square & friendly != 0 {
            ControlType::Defend
        } else if square & enemy != 0 {
            ControlType::Attack
        } else {
            ControlType::Control
        };

        let is_obscured = (square & obscured) != 0;

        controlled.push(Control {
            pos: to_pos,
            control_type,
            color: piece.color,
            direction: Some(Vector::between(piece.pos, to_pos)),
            obscured: is_obscured
        });

        rem &= rem - 1;
    }

    controlled
}

pub fn get_controlled_squares_queen(piece: &PartialPiece, board: &Board) -> Vec<Control> {
    let file = piece.pos.x;
    let rank = piece.pos.y;

    let mut controlled: Vec<Control> = Vec::with_capacity(27);

    for &dir in &QUEEN_DIRECTIONS {
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

pub fn get_pins_queen(piece: &Piece, board: &Board) -> Vec<Pin> {
    let file = piece.pos.x;
    let rank = piece.pos.y;

    let mut pins: Vec<Pin> = Vec::with_capacity(8);

    for dir in QUEEN_DIRECTIONS {
        let mut enemy_piece: Option<Piece> = None;
        let mut potential_pin = false;

        for i in 1..9 {
            let t_file = Position::clamp(file as isize + dir.x * i);
            let t_rank = Position::clamp(rank as isize + dir.y * i);

            if !Board::in_bounds(t_rank, t_file) { break };

            let other = board.get_piece_at(t_rank, t_file);
            if let Some(enemy) = other {
                if enemy.color == piece.color { break };

                if enemy.piece_type == PieceType::King {
                    if potential_pin {
                        pins.push(Pin { 
                            position: enemy_piece.as_ref().unwrap().pos,
                            to: Position { x: t_file, y: t_rank },
                            color: piece.color,
                            dir
                        })
                    }
                    break;
                } else {
                    if potential_pin {
                        break;
                    }

                    enemy_piece = Some(enemy.clone());
                    potential_pin = true;
                }
            }
        }
    }

    pins
}