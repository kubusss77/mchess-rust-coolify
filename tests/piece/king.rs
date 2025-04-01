use mchess::{board::Board, moves::MoveType, piece::PieceColor, pieces::king::get_controlled_squares_king};

use crate::common::{alg, show_mask};

#[test]
fn test_king_basic_moves() {
    let board = Board::from_fen("k7/8/8/8/3K4/8/8/8 w - - 0 1");
    let pos = alg("d4");
    let king = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(king.index);
    assert_eq!(moves.len(), 8, "King in the center should have 8 moves");
}

#[test]
fn test_king_in_check() {
    let board = Board::from_fen("k7/8/8/8/8/8/4r3/4K3 w - - 0 1");
    let pos = alg("e1");
    let king = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(king.index);
    assert_eq!(moves.len(), 3);
}

#[test]
fn test_king_castling() {
    let board = Board::from_fen("k7/8/8/8/8/8/8/R3K2R w KQ - 0 1");
    let pos = alg("e1");
    let king = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(king.index);
    assert!(moves.iter().any(|m| m.move_type.contains(&MoveType::Castling) && m.to.x == 6));
    assert!(moves.iter().any(|m| m.move_type.contains(&MoveType::Castling) && m.to.x == 2));
}

#[test]
fn test_castling_through_check() {
    let board = Board::from_fen("k7/8/8/8/8/5r2/8/R3K2R w KQ - 0 1");
    let pos = alg("e1");
    let king = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(king.index);
    assert!(moves.iter().all(|m| !(m.move_type.contains(&MoveType::Castling) && m.to.x == 6)));
}

#[test]
fn test_king_control() {
    let board = Board::from_fen("8/6k1/8/8/8/2Kp4/8/8 w - - 0 1");
    let pos = alg("c3");
    let king = board.get_piece_at(pos.y, pos.x).unwrap();
    let control = get_controlled_squares_king(&king.to_partial(), &board);
    assert_eq!(control.len(), 8);
}

#[test]
fn test_king_wrapping() {
    let board = Board::from_fen("6k1/8/8/8/8/8/8/6K1 w - - 0 1");
    let pos = alg("g1");
    let king = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(king.index);
    println!("{:?}", moves);
    assert_eq!(moves.len(), 5);
}

#[test]
fn test_king_square_freed() {
    let mut board = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");

    let moves = board.get_total_legal_moves(None);
    
    for m in &moves {
        if format!("{:?}", m) == "e2a6" {
            board.make_move(m);
            println!("{:?}", board);
            show_mask(board.bb.white_pieces);
        }
    }

    show_mask(board.control_bitboards.black_attack);

    let moves = board.get_total_legal_moves(None);
    
    for m in &moves {
        if format!("{:?}", m) == "b6c4" {
            board.make_move(m);
        }
    }

    show_mask(board.control_bitboards.black_attack);

    let pos = alg("e2");

    println!("{:?}", board.get_control_at(pos.y, pos.x, Some(PieceColor::Black), true));

    assert_eq!(board.get_total_legal_moves(None).len(), 50);
}