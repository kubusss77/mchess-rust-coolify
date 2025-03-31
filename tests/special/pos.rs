use mchess::board::Board;

use crate::common::show_mask;

#[test]
fn test_pos3_g2g3() {
    let mut board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");

    let moves = board.get_total_legal_moves(None);

    for m in &moves {
        if format!("{:?}", m) == "g2g3" {
            board.make_move(m);
        }
    }

    let moves = board.get_total_legal_moves(None);

    for m in &moves {
        if format!("{:?}", m) == "h4h3" {
            board.make_move(m);
        }
    }

    let moves = board.get_total_legal_moves(None);

    for m in &moves {
        if format!("{:?}", m) == "e2e4" {
            board.make_move(m);
        }
    }

    println!("{:?}", board);
    println!("{:?}", board.check.get(&mchess::piece::PieceColor::Black));

    assert_eq!(board.get_total_legal_moves(None).len(), 20);
}

#[test]
fn test_pos3_b4f4() {
    let mut board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");

    let moves = board.get_total_legal_moves(None);

    for m in &moves {
        if format!("{:?}", m) == "b4f4" {
            board.make_move(m);
        }
    }

    let moves = board.get_total_legal_moves(None);

    for m in &moves {
        if format!("{:?}", m) == "h4g3" {
            board.make_move(m);
        }
    }

    let moves = board.get_total_legal_moves(None);

    for m in &moves {
        if format!("{:?}", m) == "f4f3" {
            board.make_move(m);
        }
    }

    println!("{:?}", board);
    println!("{:?}", board.check.get(&mchess::piece::PieceColor::Black));

    assert_eq!(board.get_total_legal_moves(None).len(), 4);
}

#[test]
fn test_pos3_b4b3() {
    let mut board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");

    let moves = board.get_total_legal_moves(None);

    for m in &moves {
        if format!("{:?}", m) == "b4b3" {
            board.make_move(m);
        }
    }

    let moves = board.get_total_legal_moves(None);

    for m in &moves {
        if format!("{:?}", m) == "c7c5" {
            board.make_move(m);
        }
    }

    let moves = board.get_total_legal_moves(None);

    for m in &moves {
        if format!("{:?}", m) == "g2g3" {
            board.make_move(m);
        }
    }

    println!("{:?}", board);
    println!("{:?}", board.check.get(&mchess::piece::PieceColor::Black));

    assert_eq!(board.get_total_legal_moves(None).len(), 4);
}

#[test]
fn test_pos3_a5a6() {
    let mut board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");

    let moves = board.get_total_legal_moves(None);

    for m in &moves {
        if format!("{:?}", m) == "a5a6" {
            board.make_move(m);
        }
    }

    let moves = board.get_total_legal_moves(None);

    for m in &moves {
        if format!("{:?}", m) == "c7c5" {
            board.make_move(m);
        }
    }

    let moves = board.get_total_legal_moves(None);

    for m in &moves {
        if format!("{:?}", m) == "b5c6" {
            println!("{:?}\n{:?}", m.move_type, m.captured);
            board.make_move(m);
        }
    }

    println!("{:?}", board);
    show_mask(board.bb.black_pawns);
    show_mask(board.bb.all_pieces);

    println!("{:?}", board.get_total_legal_moves(None));

    assert_eq!(board.get_total_legal_moves(None).len(), 14);
}

#[test]
fn test_pos_kiwipete_c3b1() {
    let mut board = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    
    let moves = board.get_total_legal_moves(None);

    for m in &moves {
        if format!("{:?}", m) == "c3b1" {
            board.make_move(m);
        }
    }

    let moves = board.get_total_legal_moves(None);

    for m in &moves {
        if format!("{:?}", m) == "f6h7" {
            board.make_move(m);
        }
    }

    println!("{:?}", board);

    show_mask(board.bb.all_pieces);

    assert_eq!(board.get_total_legal_moves(None).len(), 50);
}