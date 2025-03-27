use mchess::board::Board;

use crate::common::alg;

#[test]
fn test_rook_basic_moves() {
    let mut board = Board::from_fen("4k3/8/8/8/3R4/8/8/4K3 w - - 0 1");
    let pos = alg("d4");
    let rook = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(rook.index);
    assert_eq!(moves.len(), 14, "Rook in the center should have 14 legal moves");
}

#[test]
fn test_diagonal_pinned_rook() {
    let mut board = Board::from_fen("4k3/8/5b2/8/8/2R5/8/K7 w - - 0 1");
    let pos = alg("c3");
    let rook = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(rook.index);
    assert_eq!(moves.len(), 0, "Diagonally pinned rook should have no legal moves");
}

#[test]
fn test_orthogonally_pinned_rook() {
    let mut board = Board::from_fen("4k3/4q3/8/8/8/8/4R3/4K3 w - - 0 1");
    let pos = alg("e2");
    let rook = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(rook.index);
    assert_eq!(moves.len(), 5, "Orthogonally pinned rook should have some legal moves");
}

#[test]
fn test_rook_check_block() {
    let mut board = Board::from_fen("4k3/8/8/8/8/8/2R5/K6q w - - 0 1");
    let pos = alg("c2");
    let rook = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(rook.index);
    assert_eq!(moves.len(), 1); 
}