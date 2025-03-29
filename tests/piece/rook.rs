use mchess::{board::Board, pieces::rook::get_controlled_squares_rook};

use crate::common::alg;

#[test]
fn test_rook_basic_moves() {
    let board = Board::from_fen("4k3/8/8/8/3R4/8/8/4K3 w - - 0 1");
    let pos = alg("d4");
    let rook = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(rook.index);
    assert_eq!(moves.len(), 14, "Rook in the center should have 14 legal moves");
}

#[test]
fn test_diagonal_pinned_rook() {
    let board = Board::from_fen("4k3/8/5b2/8/8/2R5/8/K7 w - - 0 1");
    let pos = alg("c3");
    let rook = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(rook.index);
    assert_eq!(moves.len(), 0, "Diagonally pinned rook should have no legal moves");
}

#[test]
fn test_orthogonally_pinned_rook() {
    let board = Board::from_fen("4k3/4q3/8/8/8/8/4R3/4K3 w - - 0 1");
    let pos = alg("e2");
    let rook = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(rook.index);
    assert_eq!(moves.len(), 5, "Orthogonally pinned rook should have some legal moves");
}

#[test]
fn test_rook_check_block() {
    let board = Board::from_fen("4k3/8/8/8/8/8/2R5/K6q w - - 0 1");
    let pos = alg("c2");
    let rook = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(rook.index);
    assert_eq!(moves.len(), 1); 
}

#[test]
fn test_rook_move_blocked() {
    let board = Board::from_fen("4k3/8/8/8/8/8/P7/R4K2 w - - 0 1");
    let pos = alg("a1");
    let rook = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(rook.index);
    assert_eq!(moves.len(), 4);
}

#[test]
fn test_rook_control() {
    let board = Board::from_fen("6k1/8/4n3/8/2p1R3/8/8/1K6 w - - 0 1");
    let pos = alg("e4");
    let rook = board.get_piece_at(pos.y, pos.x).unwrap();
    let control = get_controlled_squares_rook(&rook.to_partial(), &board);
    assert_eq!(control.len(), 10);
}

#[test]
fn test_rook_wrapping() {
    let board = Board::from_fen("6k1/8/8/8/3p3R/8/8/6K1 w - - 0 1");
    let pos = alg("h4");
    let rook = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(rook.index);
    assert_eq!(moves.len(), 11);
}