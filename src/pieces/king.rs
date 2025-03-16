use crate::board::{Board, Control, ControlType};
use crate::moves::{Move, MoveType, Position};
use crate::piece::Piece;

fn can_move_to(piece: &Piece, board: &Board, rank: usize, file: usize, explicit: bool) -> bool {
    if !Board::in_bounds(rank, file) { return false };
    if board.get_control_at(rank, file, Some(piece.color.opposite())).len() > 0 { return false };
    if explicit {
        board.is_empty(rank, file)
    } else {
        board.square_free(rank, file, piece.color)
    }
}

fn can_move_multifile(piece: &Piece, board: &Board, rank: usize, files: Vec<isize>) -> bool {
    let mut cond = true;
    for i in files {
        let u = Position::clamp(i);
        cond |= can_move_to(piece, board, rank, u, true);
    }
    cond
}

pub fn get_legal_moves_king(piece: &Piece, board: &mut Board) -> Vec<Move> {
    let mut moves: Vec<Move> = vec![];
    let file = piece.pos.x;
    let rank = piece.pos.y;
    
    let ifile = file as isize;
    let irank = rank as isize;

    for i in -1..=1 {
        for j in -1..=1 {
            if i == 1 && j == 1 { continue };
            let t_rank = Position::clamp(irank + i);
            let t_file = Position::clamp(ifile + j);
            let other = board.get_piece_at(t_rank, t_file);

            if can_move_to(piece, board, rank, file, false) {
                moves.push(Move {
                    from: piece.pos,
                    to: Position { x: t_file, y: t_rank },
                    captured: other.clone(),
                    move_type: vec![
                        match other {
                            Some(_) => MoveType::Capture,
                            None => MoveType::Normal
                        }
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
            to: Position { x: file, y: rank + 2 },
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
            to: Position::from(ifile, irank - 2),
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

pub fn get_controlled_squares_king(piece: Piece, board: &mut Board) -> Vec<Control> {
    let file = piece.pos.x;
    let rank = piece.pos.y;

    let mut controlled: Vec<Control> = vec![];

    for i in -1..=1 {
        for j in -1..=1 {
            if i == 0 && j == 0 { continue };
            let t_file = Position::clamp(file as isize + i);
            let t_rank = Position::clamp(rank as isize + j);

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
    }

    controlled
}
