use mchess::{board::Board, moves::MoveType, pieces::pawn::get_controlled_squares_pawn};

use crate::common::alg;

#[test]
fn test_pawn_basic_moves() {
    let board = Board::from_fen("4k3/8/8/8/8/8/P7/4K3 w - - 0 1");
    let pos = alg("a2");
    let pawn = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(pawn.index);
    assert_eq!(moves.len(), 2, "Double push");
    
    let board = Board::from_fen("4k3/8/8/8/8/P7/8/4K3 w - - 0 1");
    let pos = alg("a3");
    let pawn = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(pawn.index);
    assert_eq!(moves.len(), 1, "Single push");
}

#[test]
fn test_pawn_captures() {
    let board = Board::from_fen("4k3/8/8/8/8/1p1p4/2P5/4K3 w - - 0 1");
    let pos = alg("c2");
    let pawn = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(pawn.index);
    assert_eq!(moves.len(), 4, "2 captures and 1 push");
}

#[test]
fn test_pawn_en_passant() {
    let board = Board::from_fen("4k3/8/8/8/pP6/8/8/4K3 b - b3 0 1");
    let pos = alg("a4");
    let pawn = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(pawn.index);
    assert!(moves.iter().any(|m| m.move_type.contains(&MoveType::Capture)));
}

#[test]
fn test_pawn_en_passant_disconnect() {
    let board = Board::from_fen("4k3/8/8/8/1P4p1/8/8/4K3 b - b3 0 1");
    let pos = alg("g4");
    let pawn = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(pawn.index);
    println!("{:?}\n{:?}", moves, board);
    assert_eq!(moves.len(), 1);
}

#[test]
fn test_diagonally_pinned_pawn() {
    let board = Board::from_fen("4k3/8/8/7b/8/8/4P3/3K4 w - - 0 1");
    let pos = alg("e2");
    let pawn = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(pawn.index);
    assert_eq!(moves.len(), 0);
}

#[test]
fn test_diagonally_pinned_pawn_capture() {
    let board = Board::from_fen("4k3/8/8/7b/8/5p2/4P3/3K4 w - - 0 1");
    let pos = alg("e2");
    let pawn = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(pawn.index);
    assert_eq!(moves.len(), 3);
}

#[test]
fn test_pawn_check_block() {
    let board = Board::from_fen("4k3/8/8/7b/8/8/5P2/3K4 w - - 0 1");
    let pos = alg("f2");
    let pawn = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(pawn.index);
    assert_eq!(moves.len(), 1);

    let board = Board::from_fen("4k3/8/8/7b/8/8/6P1/3K4 w - - 0 1");
    let pos = alg("g2");
    let pawn = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(pawn.index);
    assert_eq!(moves.len(), 1);
}

#[test]
fn test_pawn_control() {
    let board = Board::from_fen("6k1/8/8/8/8/4P3/8/1K6 w - - 0 1");
    let pos = alg("e3");
    let pawn = board.get_piece_at(pos.y, pos.x).unwrap();
    let control = get_controlled_squares_pawn(&pawn.to_partial(), &board);
    assert_eq!(control.len(), 3);
}