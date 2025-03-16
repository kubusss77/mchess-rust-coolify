use crate::board::{Board, Control, ControlType};
use crate::moves::{Move, MoveType, Position};
use crate::piece::{Piece, PieceColor, PieceType};

pub fn get_legal_moves_pawn(piece: Piece, board: &mut Board) -> Vec<Move> {
    let file = piece.pos.x;
    let rank = piece.pos.y;

    let dir: isize = if piece.color == PieceColor::White { -1 } else { 1 };

    let check_info = board.check.get(&piece.color);

    if board.is_pinned(rank, file) { return vec![] };
    if check_info.is_some_and(|c| c.double_checked) { return vec![] };

    let promotion_rank = if piece.color == PieceColor::White { 7 } else { 0 };
    let initial_rank = if piece.color == PieceColor::White { 6 } else { 2 };
    
    let advanced_rank = Position::clamp(rank as isize + dir);

    let mut moves: Vec<Move> = vec![];

    if board.is_empty(advanced_rank, file) {
        if advanced_rank == promotion_rank {
            for piece_type in [ PieceType::Knight, PieceType::Bishop, PieceType::Rook, PieceType::Queen ] {
                moves.push(Move {
                    from: piece.pos,
                    to: Position { x: file, y: advanced_rank },
                    move_type: vec![ MoveType::Promotion ],
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
                move_type: vec![ MoveType::Normal ],
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
            move_type: vec![ MoveType::Normal ],
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
                    move_type: vec![ MoveType::Capture ],
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
                    move_type: vec![ MoveType::Capture ],
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

pub fn get_controlled_squares_pawn(piece: Piece, board: &mut Board) -> Vec<Control> {
    let file = piece.pos.x;
    let rank = piece.pos.y;

    let dir = if piece.color == PieceColor::White { -1 } else { 1 };

    let mut controlled: Vec<Control> = vec![];

    for square in [-1, 1] {
        let t_file = Position::clamp(file as isize + square);
        let t_rank = Position::clamp(rank as isize + dir);

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