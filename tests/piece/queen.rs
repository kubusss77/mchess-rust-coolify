use mchess::board::Board;

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