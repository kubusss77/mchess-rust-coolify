use mchess::board::Board;

use crate::common::show_mask;

#[test]
fn test_piece_capture() {
    let mut board = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");

    let moves = board.get_total_legal_moves(None);

    for m in &moves {
        if format!("{:?}", m) == "f3h3" {
            board.make_move(m);
            println!("{:?}", m.move_type);
        }
    }

    println!("{:?}", board);
    show_mask(board.bb.black_pawns);
}

#[test]
fn test_capture_persistance() {
    let mut board = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    
    let moves = board.get_total_legal_moves(None);

    for m in &moves {
        if format!("{:?}", m) == "f3h3" {
            board.make_move(m);
        }
    }

    let moves = board.get_total_legal_moves(None);
    
    for m in &moves {
        if format!("{:?}", m) == "h8h4" {
            board.make_move(m);
        }
    }

    println!("{:?}", board);
    show_mask(board.bb.black_pieces);
}