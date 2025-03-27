use mchess::board::Board;

fn perft(board: &mut Board, depth: u32) -> u64 {
    if depth == 0 { return 1; }
    
    let moves = board.get_total_legal_moves(None);
    if depth == 1 { return moves.len() as u64; }
    
    let mut nodes = 0;
    for mov in moves {
        let history = board.make_move(&mov);
        nodes += perft(board, depth - 1);
        board.unmake_move(&mov, &history);
    }
    nodes
}

#[test]
fn test_perft() {
    let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    
    let expected = [
        1,        // depth 0
        20,       // depth 1
        400,      // depth 2
        8902,     // depth 3
        197281,   // depth 4
    ];
    
    for depth in 0..expected.len() {
        let start = std::time::Instant::now();
        let result = perft(&mut board, depth as u32);
        let duration = start.elapsed();
        
        assert_eq!(result, expected[depth], "Perft failed at depth {}", depth);
        println!("Perft depth {} = {} nodes in {:?}", depth, result, duration);

        board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    }
    
    let mut board2 = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    assert_eq!(perft(&mut board2, 1), 48);
    assert_eq!(perft(&mut board2, 2), 2039);
    
    let mut board3 = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
    assert_eq!(perft(&mut board3, 1), 14);
    assert_eq!(perft(&mut board3, 2), 191);
}