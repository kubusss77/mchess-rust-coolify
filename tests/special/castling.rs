use mchess::{board::Board, moves::{Move, MoveType}, piece::PieceColor};

use crate::common::alg;

#[test]
fn test_castling_rights() {
    let mut board = Board::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");

    assert_eq!(board.castling.white, (true, true));
    assert_eq!(board.castling.black, (true, true));

    let pos_white_king = alg("e1");
    let pos_white_rook = alg("h1");

    let pos_black_rook = alg("a8");

    let white_king = board.get_piece_at(pos_white_king.y, pos_white_king.x).unwrap();
    let white_rook = board.get_piece_at(pos_white_rook.y, pos_white_rook.x).unwrap();
    let black_rook = board.get_piece_at(pos_black_rook.y, pos_black_rook.x).unwrap();

    board.make_move(&Move {
        from: alg("g1"),
        to: alg("g1"),
        move_type: vec![MoveType::Castling],
        captured: None,
        promote_to: None,
        piece_index: white_king.index,
        piece_color: white_king.color,
        piece_type: white_king.piece_type,
        with: Some(white_rook)
    });

    println!("{:?}", board);

    assert!(!board.castling.can_castle_ks(PieceColor::White));
    assert!(!board.castling.can_castle_qs(PieceColor::White));

    board.make_move(&Move {
        from: pos_black_rook,
        to: alg("c8"),
        move_type: vec![MoveType::Normal],
        captured: None,
        promote_to: None,
        piece_index: black_rook.index,
        piece_color: black_rook.color,
        piece_type: black_rook.piece_type,
        with: None
    });

    println!("{:?}", board);

    assert!(!board.castling.can_castle_qs(PieceColor::Black));

    let mut board = Board::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");

    let king_pos = alg("e1");
    let king = board.get_piece_at(king_pos.y, king_pos.x).unwrap();

    board.make_move(&Move {
        from: king_pos,
        to: alg("f1"),
        move_type: vec![MoveType::Normal],
        captured: None,
        promote_to: None,
        piece_index: king.index,
        piece_color: king.color,
        piece_type: king.piece_type,
        with: None
    });

    println!("{:?}", board);

    assert!(!board.castling.can_castle_ks(PieceColor::White));
    assert!(!board.castling.can_castle_qs(PieceColor::White));
}