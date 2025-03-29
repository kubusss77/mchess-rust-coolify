use mchess::{board::Board, moves::{Move, MoveType}};

use crate::common::{alg, show_mask};

#[test]
fn test_block_positions() {
    let mut board = Board::from_fen("rnbqkbnr/ppp1pppp/3p4/8/2P5/8/PP1PPPPP/RNBQKBNR w KQkq - 0 1");
    let pos = alg("d1");
    let queen = board.get_piece_at(pos.y, pos.x).unwrap();
    board.make_move(&Move {
        from: pos,
        to: alg("a4"),
        move_type: vec![MoveType::Normal],
        captured: None,
        promote_to: None,
        piece_index: queen.index,
        piece_color: queen.color,
        piece_type: queen.piece_type,
        with: None
    });
    let check = board.check.get(&mchess::piece::PieceColor::Black).unwrap();

    show_mask(check.block_mask);

    assert_eq!(check.block_positions.clone().unwrap_or(vec![]).len(), 4);
}

#[test]
fn test_block_moves() {
    let mut board = Board::from_fen("rnbqkbnr/ppp1pppp/3p4/8/2P5/8/PP1PPPPP/RNBQKBNR w KQkq - 0 1");
    let pos = alg("d1");
    let queen = board.get_piece_at(pos.y, pos.x).unwrap();
    board.make_move(&Move {
        from: pos,
        to: alg("a4"),
        move_type: vec![MoveType::Normal],
        captured: None,
        promote_to: None,
        piece_index: queen.index,
        piece_color: queen.color,
        piece_type: queen.piece_type,
        with: None
    });

    println!("{:?}", board.get_block_moves(mchess::piece::PieceColor::Black));

    assert_eq!(board.get_block_moves(mchess::piece::PieceColor::Black).len(), 6);
}