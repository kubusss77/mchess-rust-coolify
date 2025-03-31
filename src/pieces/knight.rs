use crate::board::{Board, Control, ControlThreat, ControlType};
use crate::moves::{Move, MoveType, Position};
use crate::piece::{PartialPiece, Piece, PieceColor};
use crate::pieces::bitboard::{AB_FILE_INV, A_FILE_INV, GH_FILE_INV, H_FILE_INV};

pub fn get_legal_moves_knight(piece: &Piece, board: &Board) -> Vec<Move> {
    let pos = piece.pos.to_bitboard();
    let mut moves = Vec::with_capacity(8);

    if board.is_pinned(piece.pos.y, piece.pos.x).is_some() { return moves };

    let check_info = board.get_check(piece.color);
    
    let mut valid_squares = !0u64;
    if check_info.double_checked != 0u64 {
        return moves;
    }
    if check_info.block_mask != 0u64 { valid_squares = check_info.block_mask; }

    let knight_moves = ((pos << 17) & A_FILE_INV) |
                       ((pos << 15) & H_FILE_INV) |
                       ((pos << 10) & AB_FILE_INV) |
                       ((pos >> 6) & AB_FILE_INV) |
                       ((pos >> 15) & A_FILE_INV) |
                       ((pos >> 17) & H_FILE_INV) |
                       ((pos << 6) & GH_FILE_INV) |
                       ((pos >> 10) & GH_FILE_INV);

    let valid_moves = knight_moves & (board.bb.empty_squares | if piece.color == PieceColor::White { board.bb.black_pieces } else { board.bb.white_pieces }) & valid_squares;

    let mut rem = valid_moves;
    let mut a = 0;
    while rem != 0 {
        a += 1;
        if a > 100 { panic!("While loop has been running for over 100 iterations"); }
        let index = rem.trailing_zeros() as usize;
        let to_pos = Position::from_bitboard(1u64 << index);

        if check_info.checked != 0u64 && check_info.block_positions.is_some() {
            let block_pos = check_info.block_positions.as_ref().unwrap();
            if !block_pos.contains(&to_pos) {
                rem &= rem - 1;
                continue;
            }
        }

        let captured = board.get_piece_at(to_pos.y, to_pos.x);
        if captured.as_ref().is_some_and(|p| p.color == piece.color) {
            rem &= rem - 1;
            continue;
        }

        moves.push(Move {
            from: piece.pos,
            to: to_pos,
            move_type: vec![
                if captured.is_some() { MoveType::Capture } else { MoveType::Normal }; 1
            ],
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

pub fn get_controlled_squares_knight(piece: &PartialPiece, board: &Board) -> Vec<Control> {
    let pos = piece.pos.to_bitboard();
    let mut controlled = Vec::with_capacity(8);

    let knight_moves = ((pos << 17) & A_FILE_INV) |
                       ((pos << 15) & H_FILE_INV) |
                       ((pos << 10) & AB_FILE_INV) |
                       ((pos >> 6) & AB_FILE_INV) |
                       ((pos >> 15) & A_FILE_INV) |
                       ((pos >> 17) & H_FILE_INV) |
                       ((pos << 6) & GH_FILE_INV) |
                       ((pos >> 10) & GH_FILE_INV);

    if knight_moves == 0 {
        return controlled;
    }

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

    let mut rem = knight_moves;
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
            threat: ControlThreat::All
        });

        rem &= rem - 1;
    }

    controlled
}