use mchess::{board::Board, moves::MoveType};

use crate::common::alg;

#[test]
fn test_king_basic_moves() {
    let mut board = Board::from_fen("k7/8/8/8/3K4/8/8/8 w - - 0 1");
    let pos = alg("d4");
    let king = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(king.index);
    assert_eq!(moves.len(), 8);
}

#[test]
fn test_king_in_check() {
    let mut board = Board::from_fen("k7/8/8/8/8/8/4r3/4K3 w - - 0 1");
    let pos = alg("e1");
    let king = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(king.index);
    assert_eq!(moves.len(), 3);
}

#[test]
fn test_king_castling() {
    let mut board = Board::from_fen("k7/8/8/8/8/8/8/R3K2R w KQ - 0 1");
    let pos = alg("e1");
    let king = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(king.index);
    assert!(moves.iter().any(|m| m.move_type.contains(&MoveType::Castling) && m.to.x == 6));
    assert!(moves.iter().any(|m| m.move_type.contains(&MoveType::Castling) && m.to.x == 2));
}

#[test]
fn test_castling_through_check() {
    let mut board = Board::from_fen("k7/8/8/8/8/5r2/8/R3K2R w KQ - 0 1");
    let pos = alg("e1");
    let king = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(king.index);
    assert!(moves.iter().all(|m| !(m.move_type.contains(&MoveType::Castling) && m.to.x == 6)));
}