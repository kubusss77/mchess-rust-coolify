use crate::board::{Board, Control, ControlThreat, ControlType};
use crate::moves::{Move, MoveType, Position};
use crate::piece::{PartialPiece, Piece, PieceColor};

use super::bitboard::{A_FILE_INV, H_FILE_INV};

pub fn get_legal_moves_king(piece: &Piece, board: &Board) -> Vec<Move> {
    let pos = piece.pos.to_bitboard();
    let mut moves = Vec::with_capacity(8);

    let king_moves = ((pos << 1) & A_FILE_INV) |
                     ((pos >> 1) & H_FILE_INV) |
                     (pos << 8) |
                     (pos >> 8) |
                     ((pos << 9) & A_FILE_INV) |
                     ((pos << 7) & H_FILE_INV) |
                     ((pos >> 7) & A_FILE_INV) |
                     ((pos >> 9) & H_FILE_INV);

    let valid_moves = king_moves & (board.bb.empty_squares | if piece.color == PieceColor::White { board.bb.black_pieces } else { board.bb.white_pieces });

    let mut rem = valid_moves;

    let mut a = 0;
    while rem != 0 {
        a += 1;
        if a > 100 { panic!("While loop has been running for over 100 iterations"); }
        let index = rem.trailing_zeros() as usize;
        let to_pos = Position::from_bitboard(1u64 << index);

        if !board.get_control_at(to_pos.y, to_pos.x, Some(piece.color.opposite()), true).is_empty() {
            rem &= rem - 1;
            continue;
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

    let is_checked = !board.get_control_at(piece.pos.y, piece.pos.x, Some(piece.color.opposite()), true).is_empty();

    let ifile = piece.pos.x as isize;

    if board.castling.can_castle_ks(piece.color) && !is_checked && can_move_multifile(piece, board, piece.pos.y, vec![ ifile + 1, ifile + 2 ]) {
        moves.push(Move {
            from: piece.pos,
            to: Position { x: piece.pos.x + 2, y: piece.pos.y },
            captured: None,
            move_type: vec![ MoveType::Castling ],
            promote_to: None,
            piece_index: piece.index,
            piece_color: piece.color,
            piece_type: piece.piece_type,
            with: board.get_piece_at(piece.pos.y, piece.pos.x + 3)
        })
    }

    if board.castling.can_castle_qs(piece.color) && !is_checked && can_move_multifile(piece, board, piece.pos.y, vec![ ifile - 1, ifile - 2 ]) && board.is_empty(piece.pos.y, Position::clamp(ifile - 3)) {
        moves.push(Move {
            from: piece.pos,
            to: Position::from(ifile - 2, piece.pos.y as isize),
            captured: None,
            move_type: vec![ MoveType::Castling ],
            promote_to: None,
            piece_index: piece.index,
            piece_color: piece.color,
            piece_type: piece.piece_type,
            with: board.get_piece_at(piece.pos.y, Position::clamp(ifile - 4))
        })
    }

    moves
}

fn can_move_to(piece: &Piece, board: &Board, rank: usize, file: usize, explicit: bool) -> bool {
    if !Board::in_bounds(rank, file) { return false };
    if board.get_control_at(rank, file, Some(piece.color.opposite()), true).len() > 0 { return false };
    if explicit {
        board.is_empty(rank, file)
    } else {
        board.square_free(rank, file, piece.color)
    }
}

fn can_move_multifile(piece: &Piece, board: &Board, rank: usize, files: Vec<isize>) -> bool {
    files.iter().all(|&i| can_move_to(piece, board, rank, Position::clamp(i), true))
}

pub fn get_controlled_squares_king(piece: &PartialPiece, board: &Board) -> Vec<Control> {
    let pos = piece.pos.to_bitboard();
    let mut controlled = Vec::with_capacity(8);

    let king_moves = ((pos << 1) & A_FILE_INV) |
                     ((pos >> 1) & H_FILE_INV) |
                     (pos << 8) |
                     (pos >> 8) |
                     ((pos << 9) & A_FILE_INV) |
                     ((pos << 7) & H_FILE_INV) |
                     ((pos >> 7) & A_FILE_INV) |
                     ((pos >> 9) & H_FILE_INV);
    
    if king_moves == 0 {
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

    let mut rem = king_moves;
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