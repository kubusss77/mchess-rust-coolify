use std::usize;

use crate::{board::{Board, ControlTableEntry, ControlType, ResultType}, r#const::*, piece::{PieceColor, PieceType}};
use crate::moves::{Move, MoveType};

#[derive(Debug, Clone, Copy)]
pub struct EvaluationResult {
    white: f64,
    black: f64
}

impl EvaluationResult {
    pub fn combine(res1: Self, res2: Self) -> Self {
        EvaluationResult {
            white: res1.white + res2.white,
            black: res1.black + res2.black
        }
    }

    pub fn to_value(&self) -> f64 {
        self.white - self.black
    }
}

pub fn evaluate(board: &mut Board) -> EvaluationResult {
    let checkmate = board.get_result();
    match checkmate {
        ResultType::WhiteCheckmate => return EvaluationResult {
            white: f64::MAX,
            black: 0.0
        },
        ResultType::BlackCheckmate => return EvaluationResult {
            white: 0.0,
            black: f64::MAX
        },
        ResultType::Draw | ResultType::Stalemate => return EvaluationResult {
            white: 0.0,
            black: 0.0
        },
        _ => ()
    }

    let pawns = evaluate_pawns(board);
    let mobility = evaluate_mobility(board);

    EvaluationResult::combine(pawns, mobility)
}

pub fn evaluate_pawns(board: &mut Board) -> EvaluationResult {
    let mut files_white: Vec<usize> = vec![0; 8];
    let mut files_black: Vec<usize> = vec![0; 8];

    let mut values = EvaluationResult {
        white: 0.0,
        black: 0.0
    };

    for pawn in board.pieces.values().filter(|p| p.piece_type == PieceType::Pawn) {
        match pawn.color {
            PieceColor::White => files_white[pawn.pos.x] += 1,
            PieceColor::Black => files_black[pawn.pos.x] += 1
        };
    }

    for i in 0..8 {
        let file_value = (-0.1 * f64::abs(i as f64 - 3.5).powf(1.75)) + 1.05;

        let file_white = files_white[i] as f64 * file_value;

        let last_file_white = if i == 0 { 0 } else { files_white[i - 1] };
        let next_file_white = if i == 7 { 0 } else { files_white[i + 1] };

        let mut penalty_white = 0.0;

        if last_file_white == 0 { penalty_white += PAWN_ISOLATION_PENALTY; }
        if next_file_white == 0 { penalty_white += PAWN_ISOLATION_PENALTY; }

        values.white += f64::min(file_white, (1.0 - penalty_white) * (1.0 / file_white));

        let file_black = files_black[i] as f64 * file_value;

        let last_file_black = if i == 0 { 0 } else { files_black[i - 1] };
        let next_file_black = if i == 7 { 0 } else { files_black[i + 1] };

        let mut penalty_black = 0.0;

        if last_file_black == 0 { penalty_black += PAWN_ISOLATION_PENALTY; }
        if next_file_black == 0 { penalty_black += PAWN_ISOLATION_PENALTY; }

        values.black += f64::min(file_black, (1.0 - penalty_black) * (1.0 / file_black));
    }

    values
}

pub fn evaluate_mobility(board: &mut Board) -> EvaluationResult {
    let mut values = EvaluationResult {
        white: 0.0,
        black: 0.0
    };

    for (index, piece) in &board.pieces {
        let default_control = vec![];
        let control_table_entry = board.control_table_lookup.get(&index).unwrap_or(&default_control);
        let value = control_table_entry.iter().filter(|c| c.1 == ControlType::Control).collect::<Vec<_>>().len() as f64 * MOBILITY_VALUE;
        match piece.color {
            PieceColor::White => values.white += value,
            PieceColor::Black => values.black += value
        }
    }
    
    values
}

pub fn evaluate_capture(m: Move) -> f64 {
    let captured_value = m.captured.expect("Captured piece expected for MoveType::Capture").piece_type.to_value() as f64;
    match m.piece_type {
        PieceType::King => captured_value * CAPTURE_VALUE,
        _ => ((m.piece_type.to_value() as f64 - captured_value) + 8.0) * CAPTURE_VALUE
    }
} 

pub fn evaluate_move(m: &Move, board: &mut Board) -> f64 {
    let types = &m.move_type;

    let mut value = 0.0;
    if types.contains(&MoveType::Capture) { value += evaluate_capture(m.clone()) };
    if types.contains(&MoveType::Promotion) { value += m.promote_to.expect("Chosen promotion piece expected for MoveType::Promotion").to_value() as f64 * PROMOTION_VALUE };
    if types.contains(&MoveType::Castling) { value += CASTLING_VALUE };
    if types.contains(&MoveType::Check) { value += CHECK_VALUE };
    // todo: implement pawn/piece development bonus

    value
}