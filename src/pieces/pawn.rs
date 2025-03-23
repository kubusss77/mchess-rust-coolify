use crate::board::{Board, Control, ControlType};
use crate::moves::{Move, MoveType, Position};
use crate::piece::{PartialPiece, Piece, PieceColor, PieceType};
use crate::pieces::bitboard::{A_FILE_INV, H_FILE_INV, RANK_3, RANK_6};

fn bitboard_to_move(piece: &Piece, pos: u64, move_type: MoveType, board: &Board, moves: &mut Vec<Move>) {
    if pos == 0 { return };

    let position = Position::from_bitboard(pos);
    let is_promotion = (piece.color == PieceColor::White && position.y == 0)
                    || (piece.color == PieceColor::Black && position.y == 7);
    let is_capture = &move_type == &MoveType::Capture;

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
        let m = Move {
            from: piece.pos,
            to: position,
            move_type: vec![move_type; 1],
            captured: if is_capture { board.get_piece_at(position.y, position.x) } else { None },
            promote_to: None,
            piece_index: piece.index,
            piece_color: piece.color,
            piece_type: piece.piece_type,
            with: None
        };
        moves.push(m);
    }
}

pub fn get_legal_moves_pawn_bitboard(piece: &Piece, board: &Board) -> Vec<Move> {
    let pos = piece.pos.to_bitboard();
    let mut moves = Vec::with_capacity(12);

    if board.is_pinned(piece.pos.y, piece.pos.x) { return moves };

    let single_push = if piece.color == PieceColor::White {
        (pos >> 8) & board.empty_squares
    } else {
        (pos << 8) & board.empty_squares
    };

    let double_push = if piece.color == PieceColor::White {
        ((single_push & RANK_3) >> 8) & board.empty_squares
    } else {
        ((single_push & RANK_6) << 8) & board.empty_squares
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

    bitboard_to_move(piece, single_push, MoveType::Normal, board, &mut moves);
    bitboard_to_move(piece, double_push, MoveType::Normal, board, &mut moves);
    bitboard_to_move(piece, left_capture, MoveType::Capture, board, &mut moves);
    bitboard_to_move(piece, right_capture, MoveType::Capture, board, &mut moves);

    if let Some(target_square) = board.target_square {
        let en_passant_pos = target_square.to_bitboard();

        let en_passant_capture = if piece.color == PieceColor::White {
            (((pos & A_FILE_INV) >> 9) | ((pos & H_FILE_INV) >> 7)) & en_passant_pos
        } else {
            (((pos & A_FILE_INV) << 7) | ((pos & H_FILE_INV) >> 9)) & en_passant_pos
        };

        bitboard_to_move(piece, en_passant_capture, MoveType::Capture, board, &mut moves);
    }

    moves
}

pub fn get_legal_moves_pawn(piece: &Piece, board: &Board) -> Vec<Move> {
    let file = piece.pos.x;
    let rank = piece.pos.y;

    let dir: isize = if piece.color == PieceColor::White { -1 } else { 1 };

    let check_info = board.check.get(&piece.color);

    if board.is_pinned(rank, file) { return Vec::with_capacity(0) };
    if check_info.is_some_and(|c| c.double_checked) { return Vec::with_capacity(0) };

    let promotion_rank = if piece.color == PieceColor::White { 7 } else { 0 };
    let initial_rank = if piece.color == PieceColor::White { 6 } else { 2 };
    
    let advanced_rank = Position::clamp(rank as isize + dir);

    let mut moves: Vec<Move> = Vec::with_capacity(12);

    if board.is_empty(advanced_rank, file) {
        if advanced_rank == promotion_rank {
            for &piece_type in &[ PieceType::Knight, PieceType::Bishop, PieceType::Rook, PieceType::Queen ] {
                moves.push(Move {
                    from: piece.pos,
                    to: Position { x: file, y: advanced_rank },
                    move_type: vec![ MoveType::Promotion; 1 ],
                    captured: None,
                    promote_to: Some(piece_type),
                    piece_index: piece.index,
                    piece_color: piece.color,
                    piece_type: piece.piece_type,
                    with: None
                });
            }
        } else {
            moves.push(Move {
                from: piece.pos,
                to: Position { x: file, y: advanced_rank },
                move_type: vec![ MoveType::Normal; 1 ],
                captured: None,
                promote_to: None,
                piece_index: piece.index,
                piece_color: piece.color,
                piece_type: piece.piece_type,
                with: None
            });
        }
    }

    if rank == initial_rank && board.is_empty(advanced_rank, file) && board.is_empty(Position::clamp(rank as isize + dir * 2), file) {
        moves.push(Move {
            from: piece.pos,
            to: Position { x: file, y: Position::clamp(rank as isize + dir * 2) },
            move_type: vec![ MoveType::Normal; 1 ],
            captured: None,
            promote_to: None,
            piece_index: piece.index,
            piece_color: piece.color,
            piece_type: piece.piece_type,
            with: None
        });
    }

    for square in [-1, 1] {
        let advanced_file = Position::clamp(file as isize + square);
        if !Board::in_bounds(advanced_rank, advanced_file) { continue };
        let other = board.get_piece_at(advanced_rank, advanced_file);
        if other.as_ref().is_some_and(|p| p.color != piece.color) {
            if advanced_rank == promotion_rank {
                for piece_type in [ PieceType::Knight, PieceType::Bishop, PieceType::Rook, PieceType::Queen ] {
                    moves.push(Move {
                        from: piece.pos,
                        to: Position { x: advanced_file, y: advanced_rank },
                        move_type: vec![ MoveType::Promotion, MoveType::Capture ],
                        captured: other.clone(),
                        promote_to: Some(piece_type),
                        piece_index: piece.index,
                        piece_color: piece.color,
                        piece_type: piece.piece_type,
                        with: None
                    });
                }
            } else {
                moves.push(Move {
                    from: piece.pos,
                    to: Position { x: advanced_file, y: advanced_rank },
                    move_type: vec![ MoveType::Capture; 1 ],
                    captured: other.clone(),
                    promote_to: None,
                    piece_index: piece.index,
                    piece_color: piece.color,
                    piece_type: piece.piece_type,
                    with: None
                });
            }
        }
        
        if board.target_square.is_some() {
            let t_rank = board.target_square.clone().unwrap().x;
            let t_file = board.target_square.clone().unwrap().y;

            if advanced_rank == t_rank && advanced_file == t_file {
                moves.push(Move {
                    from: piece.pos,
                    to: Position { x: advanced_file, y: advanced_rank },
                    move_type: vec![ MoveType::Capture; 1 ],
                    captured: board.get_piece_at(rank, advanced_file),
                    promote_to: None,
                    piece_index: piece.index,
                    piece_color: piece.color,
                    piece_type: piece.piece_type,
                    with: None
                });
            }
        }
    }

    moves
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
            obscured: false
        });
    }

    controlled
}