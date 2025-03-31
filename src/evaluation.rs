use std::usize;

use crate::{board::{Board, ResultType}, r#const::*, piece::{PartialPiece, PieceColor, PieceType}, pieces::{bitboard::{A_FILE_INV, H_FILE_INV}, queen::get_controlled_squares_queen}};

#[derive(Debug, Clone, Copy)]
pub struct EvaluationResult {
    pub white: f64,
    pub black: f64
}

impl EvaluationResult {
    pub fn combine(&self, res: Self) -> Self {
        EvaluationResult {
            white: self.white + res.white,
            black: self.black + res.black
        }
    }

    pub fn to_value(&self) -> f64 {
        self.white - self.black
    }

    pub fn default() -> Self {
        EvaluationResult {
            white: 0.0,
            black: 0.0
        }
    }
}

pub fn evaluate(board: &mut Board) -> EvaluationResult {
    let checkmate = board.get_result();
    match checkmate {
        ResultType::WhiteCheckmate => return EvaluationResult {
            white: 10000000000.0,
            black: 0.0
        },
        ResultType::BlackCheckmate => return EvaluationResult {
            white: 0.0,
            black: 10000000000.0
        },
        ResultType::Draw | ResultType::Stalemate => return EvaluationResult {
            white: 0.0,
            black: 0.0
        },
        _ => ()
    }

    let mut value = EvaluationResult::default();

    for piece in board.pieces.values() {
        if piece.piece_type == PieceType::King { continue; }
        match piece.color {
            PieceColor::White => value.white += piece.piece_type.to_value() as f64,
            PieceColor::Black => value.black += piece.piece_type.to_value() as f64
        }
    }

    let pawns = evaluate_pawns(board);
    let mobility = evaluate_mobility(board);
    let piece_safety = evaluate_piece_safety(board);
    let positions = evaluate_positions(board);
    let king_safety = evaluate_kings_safety(board);

    value.combine(pawns)
         .combine(mobility)
         .combine(piece_safety)
         .combine(positions)
         .combine(king_safety)
}

pub fn evaluate_pawns(board: &mut Board) -> EvaluationResult {
    let mut files_white: Vec<usize> = vec![0; 8];
    let mut files_black: Vec<usize> = vec![0; 8];

    let mut values = EvaluationResult::default();

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
    let mut values = EvaluationResult::default();

    for (index, piece) in &board.pieces {
        let value = board.mobility_cache.get(index).unwrap_or(&0.0);

        match piece.color {
            PieceColor::White => values.white += value,
            PieceColor::Black => values.black += value
        }
    }
    
    values
}

pub fn evaluate_piece_safety(board: &mut Board) -> EvaluationResult {
    let mut value = EvaluationResult::default();

    for piece in board.pieces.values() {
        if piece.piece_type == PieceType::King { continue; }

        let pos = piece.pos;
        let piece_value = piece.piece_type.to_value() as f64;

        let attackers = board.get_control_at(pos.y, pos.x, Some(piece.color.opposite()), true);
        let defenders = board.get_control_at(pos.y, pos.x, Some(piece.color), true);

        if !attackers.is_empty() && defenders.is_empty() {
            match piece.color {
                PieceColor::White => value.white -= piece_value * NO_SAFETY_PENALTY,
                PieceColor::Black => value.black -= piece_value * NO_SAFETY_PENALTY
            }
        } else if !attackers.is_empty() {
            let lowest_attacker_value = attackers.iter()
                .map(|a| a.origin.piece_type.to_value() as f64)
                .min_by(|a, b| a.total_cmp(b))
                .unwrap_or(0.0);
            
            if lowest_attacker_value < piece_value {
                match piece.color {
                    PieceColor::White => value.white -= (piece_value - lowest_attacker_value) * LOW_SAFETY_PENALTY,
                    PieceColor::Black => value.black -= (piece_value - lowest_attacker_value) * LOW_SAFETY_PENALTY
                }
            }
        }
    }
    
    value
}

pub fn evaluate_position(board: &Board, piece_type: PieceType, x: usize, y: usize) -> f64 {
    match piece_type {
        PieceType::Pawn => PAWN_TABLE[y][x],
        PieceType::Knight => KNIGHT_TABLE[y][x],
        PieceType::Bishop => BISHOP_TABLE[y][x],
        PieceType::Rook => ROOK_TABLE[y][x],
        PieceType::Queen => QUEEN_TABLE[y][x],
        PieceType::King => {
            let phase = board.calculate_phase();
            (KING_MIDDLEGAME_TABLE[y][x] * (1.0 - phase)) + (KING_ENDGAME_TABLE[y][x] * phase)
        }
    }
}

pub fn evaluate_positions(board: &Board) -> EvaluationResult {
    let mut value = EvaluationResult::default();

    for piece in board.pieces.values() {
        let x = piece.pos.x;
        let y = piece.pos.y;

        let y_index = if piece.color == PieceColor::White { y } else { 7 - y };

        let position_value = evaluate_position(board, piece.piece_type, x, y_index);

        match piece.color {
            PieceColor::White => value.white += position_value,
            PieceColor::Black => value.black += position_value
        };
    }

    value
}

pub fn evaluate_king_safety(board: &Board, color: PieceColor) -> f64 {
    // pawn shield
    let king = board.get_king(color).unwrap();
    let pos = king.pos.to_bitboard();

    let mask = ((pos << 1) & A_FILE_INV) |
               ((pos >> 1) & H_FILE_INV) |
               (pos << 8) |
               (pos >> 8) |
               ((pos << 9) & A_FILE_INV) |
               ((pos << 7) & H_FILE_INV) |
               ((pos >> 7) & A_FILE_INV) |
               ((pos >> 9) & H_FILE_INV);
    
    let shield = if color == PieceColor::White {
        let north = (pos << 16) |
                    ((pos << 17) & A_FILE_INV) |
                    ((pos << 15) & H_FILE_INV);
        
        mask | north
    } else {
        let south = (pos >> 16) |
                    ((pos >> 17) & A_FILE_INV) |
                    ((pos >> 15) & H_FILE_INV);
        
        mask | south
    };
    
    let pawns = if color == PieceColor::White {
        board.bb.white_pawns
    } else {
        board.bb.black_pawns
    };

    let positions = shield & pawns;

    let shield_value = (positions.count_ones() as f64) * PAWN_SHIELD_VALUE;

    let breathing_penalty = if (mask & pawns).count_ones() >= 3 {
        BREATHING_PENALTY
    } else {
        0.0
    };

    // pawn storm
    let enemy_pawns = if color == PieceColor::White {
        board.bb.black_pawns
    } else {
        board.bb.white_pawns
    };

    let storm_penalty = if color == PieceColor::White {
        let zone1 = ((pos <<  8) | ((pos <<  9) & A_FILE_INV) | ((pos <<  7) & H_FILE_INV)) & enemy_pawns;
        let zone2 = ((pos << 16) | ((pos << 17) & A_FILE_INV) | ((pos << 15) & H_FILE_INV)) & enemy_pawns;
        let zone3 = ((pos << 24) | ((pos << 25) & A_FILE_INV) | ((pos << 23) & H_FILE_INV)) & enemy_pawns;

        (zone1.count_ones() * 3 + zone2.count_ones() * 2 + zone3.count_ones()) as f64
    } else {
        let zone1 = ((pos >>  8) | ((pos >>  9) & A_FILE_INV) | ((pos >>  7) & H_FILE_INV)) & enemy_pawns;
        let zone2 = ((pos >> 16) | ((pos >> 17) & A_FILE_INV) | ((pos >> 15) & H_FILE_INV)) & enemy_pawns;
        let zone3 = ((pos >> 24) | ((pos >> 25) & A_FILE_INV) | ((pos >> 23) & H_FILE_INV)) & enemy_pawns;

        (zone1.count_ones() * 3 + zone2.count_ones() * 2 + zone3.count_ones()) as f64
    } * PAWN_STORM_PENALTY;

    // virtual mobility
    let mobility_penalty = (get_controlled_squares_queen(&PartialPiece {
        piece_type: PieceType::Queen,
        pos: king.pos,
        color: king.color
    }, &board).len() as f64) * VIRTUAL_MOBILITY_PENALTY;

    // attack penalty
    let mut attacks = 0.0;
    let mut rem = shield;
    while rem != 0 {
        let index = rem.trailing_zeros();
        let square = 1u64 << index;

        if let Some(entries) = board.control_bitboards.control_entries.get(&square) {
            attacks += entries.len() as f64;
        }

        rem &= rem - 1;
    }
    let attack_penalty = attacks * ATTACK_PENALTY;

    // position value
    let shift = if king.color == PieceColor::White {
        63.0 - pos.trailing_zeros() as f64
    } else {
        pos.trailing_zeros() as f64
    };
    let log_scale = (64.0_f64).log10();
    let position_value = (64.0 - 0.5 * shift.powf(1.15)).log10() / log_scale;
    let scaled_position_value = (position_value * 5.0) - 3.5;

    let safety_score = shield_value + scaled_position_value - breathing_penalty - storm_penalty - mobility_penalty - attack_penalty;

    let attack_potential = if king.color == PieceColor::White {
        let queens = (board.bb.black_queens.count_ones() as f64) * 3.0;
        let rooks = (board.bb.black_rooks.count_ones() as f64) * 2.0;
        let bishops = (board.bb.black_bishops.count_ones() as f64) * 1.5;
        let knights = (board.bb.black_knights.count_ones() as f64) * 1.5;
        
        queens + rooks + bishops + knights
    } else {
        let queens = (board.bb.white_queens.count_ones() as f64) * 3.0;
        let rooks = (board.bb.white_rooks.count_ones() as f64) * 2.0;
        let bishops = (board.bb.white_bishops.count_ones() as f64) * 1.5;
        let knights = (board.bb.white_knights.count_ones() as f64) * 1.5;
        
        queens + rooks + bishops + knights
    };

    const MAX_ATTACK_POTENTIAL: f64 = 13.0;

    let scale_factor = attack_potential / MAX_ATTACK_POTENTIAL;

    const MIN_SCALE: f64 = 0.2;
    let scale = scale_factor.max(MIN_SCALE);

    if safety_score >= 0.0 {
        safety_score
    } else {
        safety_score * scale
    }
}

pub fn evaluate_kings_safety(board: &Board) -> EvaluationResult {
    let white = evaluate_king_safety(board, PieceColor::White) * KING_SAFETY_FACTOR;
    let black = evaluate_king_safety(board, PieceColor::Black) * KING_SAFETY_FACTOR;

    EvaluationResult { 
        white, 
        black 
    }
}