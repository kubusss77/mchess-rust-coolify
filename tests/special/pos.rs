use mchess::{board::Board, piece::PieceColor};

use crate::common::{alg, show_mask};

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
    println!("{:?}", board.get_check(PieceColor::Black));

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
    println!("{:?}", board.get_check(PieceColor::Black));

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
    println!("{:?}", board.get_check(PieceColor::Black));

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
fn test_pos3_g2g4() {
    let mut board = Board::from_fen("8/2p5/3p4/KP5r/1R3pPk/8/4P3/8 b - - 0 1");
    
    let moves = board.get_total_legal_moves(None);
    
    for m in &moves {
        if format!("{:?}", m) == "g2g4" {
            board.make_move(m);
        }
    }

    let moves = board.get_total_legal_moves(None);

    for m in &moves {
        if format!("{:?}", m) == "h4g5" {
            board.make_move(m);
        }
    }

    let moves = board.get_total_legal_moves(None);

    for m in &moves {
        if format!("{:?}", m) == "b5b6" {
            board.make_move(m);
        }
    }

    let moves = board.get_total_legal_moves(None);

    for m in &moves {
        if format!("{:?}", m) == "g5h4" {
            board.make_move(m);
        }
    }

    show_mask(board.get_check(PieceColor::White).block_mask);
    show_mask(board.bb.black_pieces);

    let pos = alg("g5");

    if let Some(entries) = board.control_bitboards.control_entries.get(&pos.to_bitboard()) {
        println!("{:?}", entries);
    }

    println!("{:?}", board.get_control_at(pos.y, pos.x, Some(PieceColor::White), false));

    let pos = alg("g4");
    let pawn = board.get_piece_at(pos.y, pos.x).unwrap();

    println!("{:?}", board.get_controlled_squares(pawn.index));

    println!("{:?}", board.get_total_legal_moves(None));

    assert_eq!(board.get_total_legal_moves(None).len(), 5);
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

#[test]
fn test_pos5() {
    let mut board = Board::from_fen("rnb2k1r/pp1Pbppp/2p5/q7/2B5/2P5/PP2NnPP/RNBQK2R w KQ - 1 9");

    let moves = board.get_total_legal_moves(None);
    
    for m in &moves {
        if format!("{:?}", m) == "d7c8q" {
            board.make_move(m);
        }
    }

    println!("{:?}", board);

    show_mask(board.black_check.block_mask);

    assert_eq!(board.get_total_legal_moves(None).len(), 2);
}

#[test]
fn test_check() {
    let mut board = Board::from_fen("2k2r2/1ppp4/pnQ5/8/8/8/3q1PPP/6K1 w - - 0 1");
    
    let moves = board.get_total_legal_moves(None);

    for m in &moves {
        if format!("{:?}", m) == "c6c7" {
            let history = board.make_move(m);
            println!("{:?}", board);
        
            show_mask(board.black_check.block_mask);
        
            assert_eq!(board.get_total_legal_moves(None).len(), 1);
            
            board.unmake_move(m, &history);

            println!("{:?}", board);
        }
    }

}