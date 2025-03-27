use mchess::{board::Board, piece::PieceColor};

use crate::common::{alg, show_mask};

#[test]
fn test_knight_basic_moves() {
    let mut board = Board::from_fen("4k3/8/8/8/3N4/8/8/4K3 w - - 0 1");
    let pos = alg("d4");
    let knight = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(knight.index);
    assert_eq!(moves.len(), 8); 
}

#[test]
fn test_pinned_knight() {
    let mut board = Board::from_fen("4k3/4r3/8/8/4N3/8/8/4K3 w - - 0 1");
    board.update_pins();
    let pos = alg("e4");
    let knight = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(knight.index);
    assert_eq!(moves.len(), 0);
}

#[test]
fn test_knight_check_block() {
    let mut board = Board::from_fen("4k3/8/8/7b/3N4/8/8/3K4 w - - 0 1");
    let pos = alg("d4");
    let knight = board.get_piece_at(pos.y, pos.x).unwrap();
    board.check_control_all();
    let moves = board.get_legal_moves(knight.index);
    println!("{:?}\n{:?}", moves, board);
    show_mask(board.all_pieces);
    show_mask(board.check.get(&PieceColor::White).unwrap().block_mask);
    assert_eq!(moves.len(), 2);
}