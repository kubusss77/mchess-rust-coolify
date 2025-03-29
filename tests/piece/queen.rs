use mchess::{board::Board, pieces::queen::get_controlled_squares_queen_bitboard};

use crate::common::alg;

#[test]
fn test_queen_basic_moves() {
    let mut board = Board::from_fen("4k3/8/8/8/3Q4/8/8/4K3 w - - 0 1");
    let pos = alg("d4");
    let queen = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(queen.index);
    assert_eq!(moves.len(), 27, "Queen in the center should have 27 legal moves");
}

#[test]
fn test_diagonal_pinned_queen() {
    let mut board = Board::from_fen("4k3/6b1/8/8/8/2Q5/8/K7 w - - 0 1");
    let pos = alg("c3");
    let queen = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(queen.index);
    assert_eq!(moves.len(), 5, "Diagonally pinned queen should have some legal moves"); 
}

#[test]
fn test_orthogonal_pinned_queen() {
    let mut board = Board::from_fen("4k3/4r3/8/8/8/8/4Q3/4K3 w - - 0 1");
    let pos = alg("e2");
    let queen = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(queen.index);
    assert_eq!(moves.len(), 5, "Orthogonally pinned queen should have some legal moves"); 
}

#[test]
fn test_queen_check_block() {
    let mut board = Board::from_fen("4k3/8/8/8/8/8/3Q4/K6r w - - 0 1");
    let pos = alg("d2");
    let queen = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(queen.index);
    assert_eq!(moves.len(), 3); 
}

#[test]
fn test_queen_control() {
    let board = Board::from_fen("6k1/8/4n3/8/2p1Q3/8/6p1/1K6 w - - 0 1");
    let pos = alg("e4");
    let queen = board.get_piece_at(pos.y, pos.x).unwrap();
    let control = get_controlled_squares_queen_bitboard(&queen.to_partial(), &board);
    assert_eq!(control.len(), 22);
}