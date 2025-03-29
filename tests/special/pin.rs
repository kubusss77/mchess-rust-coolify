use mchess::{board::Board, moves::{Move, MoveType}};

use crate::common::alg;

#[test]
fn test_pin_detection() {
    let mut board = Board::from_fen("2k5/3p4/8/5B2/8/8/8/4K3 w - - 0 1");
    
    let pos = alg("d7");
    
    let pin = board.is_pinned(pos.y, pos.x);
    assert!(pin.is_some(), "Pawn should be pinned");
    
    if let Some(direction) = pin {
        assert_eq!(direction.x, -1);
        assert_eq!(direction.y, -1);
    }
    
    let pawn = board.get_piece_at(pos.y, pos.x).unwrap();
    let moves = board.get_legal_moves(pawn.index);
    assert_eq!(moves.len(), 0, "Pinned pawn should have no legal moves");
}

#[test]
fn test_phantom_pins() {
    let mut board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
    let pos = alg("e2");
    let white_pawn = board.get_piece_at(pos.y, pos.x).unwrap();
    let pos2 = alg("f4");
    let black_pawn = board.get_piece_at(pos2.y, pos2.x).unwrap();

    board.make_move(&Move {
        from: pos,
        to: alg("e4"),
        move_type: vec![MoveType::Normal],
        captured: None,
        promote_to: None,
        piece_index: white_pawn.index,
        piece_color: white_pawn.color,
        piece_type: white_pawn.piece_type,
        with: None
    });

    assert_eq!(board.target_piece, white_pawn.index as i32);
    assert_eq!(board.get_legal_moves(black_pawn.index).len(), 1);
    assert!(board.is_phantom_pinned(pos2.y, pos2.x));
}