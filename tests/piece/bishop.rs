use mchess::board::Board;

use crate::common::alg;

#[test]
fn test_bishop_basic_moves() {
    let mut board = Board::from_fen("4k3/8/8/8/3B4/8/8/4K3 w - - 0 1");
    let pos = alg("d4");
    let bishop = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(bishop.index);
    assert_eq!(moves.len(), 13);
}

#[test]
fn test_diagonal_pinned_bishop() {
    let mut board = Board::from_fen("4k3/8/5q2/8/8/2B5/8/K7 w - - 0 1");
    let pos = alg("c3");
    let bishop = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(bishop.index);
    assert_eq!(moves.len(), 4); 
}

#[test]
fn test_orthogonal_pinned_bishop() {
    let mut board = Board::from_fen("4k3/8/2q5/8/8/2B5/8/2K5 w - - 0 1");
    let pos = alg("c3");
    let bishop = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(bishop.index);
    assert_eq!(moves.len(), 0);
}

#[test]
fn test_bishop_check_block() {
    let mut board = Board::from_fen("4k3/8/8/8/8/3B4/8/K6q w - - 0 1");
    let pos = alg("d3");
    let bishop = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(bishop.index);
    assert_eq!(moves.len(), 2); 
}