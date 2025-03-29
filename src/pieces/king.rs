use crate::board::{Board, Control, ControlThreat, ControlType};
use crate::moves::{Move, MoveType, Position};
use crate::piece::{PartialPiece, Piece, PieceColor};

use super::bitboard::{A_FILE_INV, H_FILE_INV};

pub fn get_legal_moves_king_bitboard(piece: &Piece, board: &Board) -> Vec<Move> {
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

    let valid_moves = king_moves & (board.empty_squares | if piece.color == PieceColor::White { board.black_pieces } else { board.white_pieces });

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

    let ifile = piece.pos.x as isize;

    if board.castling.can_castle_ks(piece.color) && can_move_multifile(piece, board, piece.pos.y, vec![ ifile + 1, ifile + 2 ]) {
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

    if board.castling.can_castle_qs(piece.color) && can_move_multifile(piece, board, piece.pos.y, vec![ ifile - 1, ifile - 2 ]) {
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

pub fn get_legal_moves_king(piece: &Piece, board: &Board) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::with_capacity(8);
    let file = piece.pos.x;
    let rank = piece.pos.y;
    
    let ifile = file as isize;
    let irank = rank as isize;

    for i in -1..=1 {
        for j in -1..=1 {
            if i == 0 && j == 0 { continue };
            let t_rank = Position::clamp(irank + i);
            let t_file = Position::clamp(ifile + j);
            let other = board.get_piece_at(t_rank, t_file);

            if can_move_to(piece, board, t_rank, t_file, false) {
                moves.push(Move {
                    from: piece.pos,
                    to: Position { x: t_file, y: t_rank },
                    captured: other.clone(),
                    move_type: vec![
                        match other {
                            Some(_) => MoveType::Capture,
                            None => MoveType::Normal
                        }; 1
                    ],
                    promote_to: None,
                    piece_index: piece.index,
                    piece_color: piece.color,
                    piece_type: piece.piece_type,
                    with: None
                })
            }
        }
    }

    if board.castling.can_castle_ks(piece.color) && can_move_multifile(piece, board, rank, vec![ ifile + 1, ifile + 2 ]) {
        moves.push(Move {
            from: piece.pos,
            to: Position { x: file + 2, y: rank },
            captured: None,
            move_type: vec![ MoveType::Castling ],
            promote_to: None,
            piece_index: piece.index,
            piece_color: piece.color,
            piece_type: piece.piece_type,
            with: board.get_piece_at(rank, file + 3)
        })
    }

    if board.castling.can_castle_qs(piece.color) && can_move_multifile(piece, board, rank, vec![ ifile - 1, ifile - 2, ifile - 3 ]) {
        moves.push(Move {
            from: piece.pos,
            to: Position::from(ifile - 2, irank),
            captured: None,
            move_type: vec![ MoveType::Castling ],
            promote_to: None,
            piece_index: piece.index,
            piece_color: piece.color,
            piece_type: piece.piece_type,
            with: board.get_piece_at(rank, Position::clamp(ifile - 4))
        })
    }

    moves
}

pub fn get_controlled_squares_king_bitboard(piece: &PartialPiece, board: &Board) -> Vec<Control> {
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
        board.white_pieces
    } else {
        board.black_pieces
    };

    let enemy = if piece.color == PieceColor::White {
        board.black_pieces
    } else {
        board.white_pieces
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

pub fn get_controlled_squares_king(piece: &PartialPiece, board: &Board) -> Vec<Control> {
    let file = piece.pos.x;
    let rank = piece.pos.y;

    let mut controlled: Vec<Control> = Vec::with_capacity(8);

    for i in -1..=1 {
        for j in -1..=1 {
            if i == 0 && j == 0 { continue };
            let t_file = Position::clamp(file as isize + i);
            let t_rank = Position::clamp(rank as isize + j);

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
                threat: ControlThreat::All
            });
        }
    }

    controlled
}
