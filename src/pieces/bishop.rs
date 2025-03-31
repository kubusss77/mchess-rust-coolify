use crate::board::{Board, Control, ControlThreat, ControlType};
use crate::moves::{Move, MoveType, Pin, Position, Vector};
use crate::piece::{PartialPiece, Piece, PieceColor, PieceType};

use super::bitboard::{A_FILE_INV, H_FILE_INV, RANK_1, RANK_8};

pub const BISHOP_DIRECTIONS: [Vector; 4] = [Vector { x: -1, y: -1 }, Vector { x: -1, y: 1 }, Vector { x: 1, y: -1 }, Vector { x: 1, y: 1}];

pub fn generate_bishop_rays(pos: u64, occupied: u64, enemy_king: u64, let_through: bool) -> (u64, u64) {
    let mut attacks = 0u64;
    let mut obscured = 0u64;
    let mut found_king = false;

    let mut ray = pos;
    while (ray & H_FILE_INV) != 0 && (ray & RANK_1) == 0 {
        ray <<= 9;
        attacks |= ray;

        if ray & enemy_king != 0 {
            found_king = true;
        } else if found_king {
            obscured |= ray;
        }

        if ray & occupied != 0 {
            if ray & enemy_king != 0 {
                found_king = true;
                if !let_through { break; }
            } else {
                break;
            }
        }
    }

    found_king = false;
    ray = pos;
    while (ray & A_FILE_INV) != 0 && (ray & RANK_1) == 0 {
        ray <<= 7;
        attacks |= ray;

        if ray & enemy_king != 0 {
            found_king = true;
        } else if found_king {
            obscured |= ray;
        }

        if ray & occupied != 0 {
            if ray & enemy_king != 0 {
                found_king = true;
                if !let_through { break; }
            } else {
                break;
            }
        }
    }

    found_king = false;
    ray = pos;
    while (ray & H_FILE_INV) != 0 && (ray & RANK_8) == 0 {
        ray >>= 7;
        attacks |= ray;

        if ray & enemy_king != 0 {
            found_king = true;
        } else if found_king {
            obscured |= ray;
        }

        if ray & occupied != 0 {
            if ray & enemy_king != 0 {
                found_king = true;
                if !let_through { break; }
            } else {
                break;
            }
        }
    }

    found_king = false;
    ray = pos;
    while (ray & A_FILE_INV) != 0 && (ray & RANK_8) == 0 {
        ray >>= 9;
        attacks |= ray;

        if ray & enemy_king != 0 {
            found_king = true;
        } else if found_king {
            obscured |= ray;
        }

        if ray & occupied != 0 {
            if ray & enemy_king != 0 {
                found_king = true;
                if !let_through { break; }
            } else {
                break;
            }
        }
    }

    (attacks, obscured)
}

pub fn get_legal_moves_bishop(piece: &Piece, board: &Board) -> Vec<Move> {
    let pos = piece.pos.to_bitboard();
    let mut moves = Vec::with_capacity(13);

    let pin_dir = board.is_pinned(piece.pos.y, piece.pos.x);
    let check_info = board.check.get(&piece.color);
    
    let mut valid_squares = !0u64;
    if let Some(check_info) = check_info {
        if check_info.double_checked != 0u64 {
            return moves;
        }
        if check_info.block_mask != 0u64 { valid_squares = check_info.block_mask; }
    }

    let (attacks, _) = generate_bishop_rays(pos, board.bb.all_pieces, 0u64, false);

    let enemy = if piece.color == PieceColor::White {
        board.bb.black_pieces
    } else {
        board.bb.white_pieces
    };

    let valid_moves = attacks & (board.bb.empty_squares | enemy) & valid_squares;

    let mut rem = valid_moves;
    let mut a = 0;
    while rem != 0 {
        a += 1;
        if a > 100 { panic!("While loop has been running for over 100 iterations"); }
        let index = rem.trailing_zeros() as usize;
        let square = 1u64 << index;
        let to_pos = Position::from_bitboard(square);

        if let Some(pin) = pin_dir {
            if pin.x != 0 && pin.y != 0 {
                let x_diff = (to_pos.x as isize - piece.pos.x as isize).signum();
                let y_diff = (to_pos.y as isize - piece.pos.y as isize).signum();

                let vec = Vector { x: x_diff, y: y_diff };

                if !vec.is_parallel_to(pin) {
                    rem &= rem - 1;
                    continue;
                }
            } else {
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

pub fn get_controlled_squares_bishop(piece: &PartialPiece, board: &Board) -> Vec<Control> {
    let pos = piece.pos.to_bitboard();
    let mut controlled = Vec::with_capacity(13);

    let (attacks, obscured) = generate_bishop_rays(pos, board.bb.all_pieces, if piece.color == PieceColor::White { board.bb.black_king } else { board.bb.white_king }, true);
    
    let friendly = if piece.color == PieceColor::White {
        board.bb.white_pieces
    } else {
        board.bb.black_pieces
    };
    
    let enemy = if piece.color == PieceColor::White {
        board.bb.black_pieces
    } else {
        board.bb.white_pieces
    };
    
    let mut rem = attacks;
    let mut a = 0;
    while rem != 0 {
        a += 1;
        if a > 100 { panic!("While loop has been running for over 100 iterations"); }
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
            obscured: is_obscured,
            threat: ControlThreat::All
        });

        rem &= rem - 1;
    }

    controlled
}

pub fn get_pins_bishop(piece: &Piece, board: &Board) -> Vec<Pin> {
    let file = piece.pos.x;
    let rank = piece.pos.y;

    let mut pins: Vec<Pin> = Vec::with_capacity(4);

    for dir in BISHOP_DIRECTIONS {
        let mut enemy_piece: Option<Piece> = None;
        let mut potential_pin = false;
        let mut is_phantom = false;
        
        for i in 1..9 {
            let t_file = Position::clamp(file as isize + dir.x * i);
            let t_rank = Position::clamp(rank as isize + dir.y * i);

            if !Board::in_bounds(t_rank, t_file) { break };

            let other = board.get_piece_at(t_rank, t_file);
            if let Some(enemy) = other {
                if enemy.color == piece.color {
                    if board.target_piece == enemy.index as i32 {
                        is_phantom = true;
                        continue;
                    } else {
                        break;
                    }
                };

                if enemy.piece_type == PieceType::King {
                    if potential_pin {
                        pins.push(Pin { 
                            position: enemy_piece.as_ref().unwrap().pos,
                            to: Position { x: t_file, y: t_rank },
                            color: piece.color,
                            dir,
                            is_phantom
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