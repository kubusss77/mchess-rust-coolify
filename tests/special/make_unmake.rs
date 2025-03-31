use mchess::board::Board;

#[test]
fn test_make_unmake_consistency() {
    let original = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let mut board = original.clone();
    
    let moves = board.get_total_legal_moves(None);
    for m in moves {
        let history = board.make_move(&m);
        board.unmake_move(&m, &history);
        
        assert_eq!(board.hash, original.hash, "Hash mismatch after unmake_move");
        assert_eq!(board.bb.white_pieces, original.bb.white_pieces, "White pieces mismatch");
        assert_eq!(board.bb.black_pieces, original.bb.black_pieces, "Black pieces mismatch");
        assert_eq!(board.turn, original.turn, "Turn mismatch");
    }
}