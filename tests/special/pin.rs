use mchess::board::Board;

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