use std::usize;

use crate::{board::{Board, ResultType}, r#const::PAWN_ISOLATION_PENALTY, piece::{PieceColor, PieceType}};

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
}

pub fn evaluate(board: &mut Board) -> EvaluationResult {
    let checkmate = board.is_checkmate();
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
    todo!()
}