use crate::board::{Board, Control, ControlThreat, ControlType};
use crate::moves::{Move, MoveType, Position, Vector};
use crate::piece::{PartialPiece, Piece, PieceColor, PieceType};
use crate::pieces::bitboard::{A_FILE_INV, H_FILE_INV, RANK_2, RANK_7};

fn bitboard_to_move(piece: &Piece, pos: u64, move_type: MoveType, board: &Board, moves: &mut Vec<Move>, pin_dir: Option<Vector>) {
    if pos == 0 { return };

    let position = Position::from_bitboard(pos);
    let is_promotion = (piece.color == PieceColor::White && position.y == 0)
                    || (piece.color == PieceColor::Black && position.y == 7);
    let is_capture = &move_type == &MoveType::Capture;

    if let Some(pin) = pin_dir {
        if pin.x == 0 && piece.pos.x != position.x {
            return;
        }
        if pin.x != 0 && pin.y != 0 {
            let x_diff = (position.x as isize - piece.pos.x as isize).signum();
            let y_diff = (position.y as isize - piece.pos.y as isize).signum();

            let vec = Vector { x: x_diff, y: y_diff };

            if !vec.is_parallel_to(pin) {
                return;
            }
        }
    }

    if is_promotion {
        for &promotion_type in &[PieceType::Queen, PieceType::Rook, PieceType::Bishop, PieceType::Knight] {
            let m = Move {
                from: piece.pos,
                to: position,
                move_type: vec![move_type, MoveType::Promotion],
                captured: if is_capture { board.get_piece_at(position.y, position.x) } else { None },
                promote_to: Some(promotion_type),
                piece_index: piece.index,
                piece_color: piece.color,
                piece_type: piece.piece_type,
                with: None
            };
            moves.push(m);
        }
    } else {
        let is_en_passant = is_capture && board.is_empty(position.y, position.x);

        let captured = if is_en_passant {
            if piece.color == PieceColor::White {
                board.get_piece_at(position.y - 1, position.x)
            } else {
                board.get_piece_at(position.y + 1, position.x)
            }
        } else if is_capture {
            board.get_piece_at(position.y, position.x)
        } else {
            None
        };

        let m = Move {
            from: piece.pos,
            to: position,
            move_type: vec![move_type; 1],
            captured,
            promote_to: None,
            piece_index: piece.index,
            piece_color: piece.color,
            piece_type: piece.piece_type,
            with: None
        };
        moves.push(m);
    }
}

pub fn get_legal_moves_pawn(piece: &Piece, board: &Board) -> Vec<Move> {
    let pos = piece.pos.to_bitboard();
    let mut moves = Vec::with_capacity(12);

    let pin_dir = board.is_pinned(piece.pos.y, piece.pos.x);
    let check_info = board.check.get(&piece.color);
    
    let mut valid_squares = !0u64;
    if let Some(check_info) = check_info {
        if check_info.double_checked != 0u64 {
            return moves;
        }
        if check_info.block_mask != 0u64 { valid_squares = check_info.block_mask; }
    }

    if let Some(pin) = pin_dir {
        if pin.x != 0 && pin.y == 0 {
            return moves;
        }
    }

    let single_push = if piece.color == PieceColor::White {
        (pos >> 8) & board.empty_squares
    } else {
        (pos << 8) & board.empty_squares
    };

    let double_push = if piece.color == PieceColor::White {
        if (pos & RANK_2) != 0 {
            ((pos >> 8) >> 8) & board.empty_squares & (single_push >> 8)
        } else {
            0
        }
    } else {
        if (pos & RANK_7) != 0 {
            ((pos << 8) << 8) & board.empty_squares & (single_push << 8)
        } else {
            0
        }
    };

    let left_capture = if piece.color == PieceColor::White {
        ((pos & A_FILE_INV) >> 9) & board.black_pieces
    } else {
        ((pos & A_FILE_INV) << 7) & board.white_pieces
    };

    let right_capture = if piece.color == PieceColor::White {
        ((pos & H_FILE_INV) >> 7) & board.black_pieces
    } else {
        ((pos & H_FILE_INV) << 9) & board.white_pieces
    };

    bitboard_to_move(piece, single_push & valid_squares, MoveType::Normal, board, &mut moves, pin_dir);
    bitboard_to_move(piece, double_push & valid_squares, MoveType::Normal, board, &mut moves, pin_dir);
    bitboard_to_move(piece, left_capture & valid_squares, MoveType::Capture, board, &mut moves, pin_dir);
    bitboard_to_move(piece, right_capture & valid_squares, MoveType::Capture, board, &mut moves, pin_dir);

    if let Some(target_square) = board.target_square {
        let en_passant_pos = target_square.to_bitboard();
        
        let en_passant_capture = if piece.color == PieceColor::White {
            (((pos & A_FILE_INV) >> 9) | ((pos & H_FILE_INV) >> 7)) & en_passant_pos
        } else {
            (((pos & A_FILE_INV) << 7) | ((pos & H_FILE_INV) << 9)) & en_passant_pos
        } & valid_squares;

        if !board.is_phantom_pinned(piece.pos.y, piece.pos.x) {
            bitboard_to_move(piece, en_passant_capture, MoveType::Capture, board, &mut moves, pin_dir);
        }
    }

    moves
}

pub fn get_controlled_squares_pawn_bitboard(piece: &PartialPiece, board: &Board) -> Vec<Control> {
    let pos = piece.pos.to_bitboard();
    let mut controlled = Vec::with_capacity(2);

    let left_capture = if piece.color == PieceColor::White {
        (pos & A_FILE_INV) >> 9
    } else {
        (pos & A_FILE_INV) << 7
    };

    let right_capture = if piece.color == PieceColor::White {
        (pos & H_FILE_INV) >> 7
    } else {
        (pos & H_FILE_INV) << 9
    };

    let single_push = if piece.color == PieceColor::White {
        (pos >> 8) & board.empty_squares
    } else {
        (pos << 8) & board.empty_squares
    };

    let double_push = if piece.color == PieceColor::White {
        if (pos & RANK_2) != 0 {
            ((pos >> 8) >> 8) & board.empty_squares & (single_push >> 8)
        } else {
            0
        }
    } else {
        if (pos & RANK_7) != 0 {
            ((pos << 8) << 8) & board.empty_squares & (single_push << 8)
        } else {
            0
        }
    };

    let attacks = left_capture | right_capture;
    let other = single_push | double_push;

    let moves = attacks | other;

    if moves == 0 {
        return controlled;
    }

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

    let mut rem = moves;
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

        controlled.push(Control {
            pos: to_pos,
            control_type,
            color: piece.color,
            direction: None,
            obscured: false,
            threat: if square & attacks != 0 { ControlThreat::Threatning } else { ControlThreat::PotentialMove }
        });

        rem &= rem - 1;
    }

    controlled
}

pub fn get_controlled_squares_pawn(piece: &PartialPiece, board: &Board) -> Vec<Control> {
    let file = piece.pos.x;
    let rank = piece.pos.y;

    let dir = if piece.color == PieceColor::White { -1 } else { 1 };

    let mut controlled: Vec<Control> = Vec::with_capacity(2);

    for square in [-1, 1] {
        let t_file = Position::clamp(file as isize + square);
        let t_rank = Position::clamp(rank as isize + dir);

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
            direction: None,
            obscured: false,
            threat: ControlThreat::Threatning
        });
    }

    controlled
}