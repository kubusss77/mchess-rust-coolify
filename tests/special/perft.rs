use mchess::board::Board;

fn perft(board: &mut Board, depth: u32) -> u64 {
    if depth == 0 { return 1; }
    
    let moves = board.get_total_legal_moves(None);
    if depth == 1 { return moves.len() as u64; }
    
    let mut nodes = 0;
    for m in moves {
        let history = board.make_move(&m);
        nodes += perft(board, depth - 1);
        board.unmake_move(&m, &history);
    }
    nodes
}

fn split_perft(board: &mut Board, depth: u32) -> u64 {
    if depth == 0 { return 1; }
    
    let moves = board.get_total_legal_moves(None);
    if depth == 1 { 
        for m in &moves {
            println!("{:?}: 1", m);
        }
        return moves.len() as u64; 
    }
    
    let mut total_nodes = 0;
    
    for m in moves {
        let move_str = format!("{:?}", m);
        
        let history = board.make_move(&m);
        let nodes = 
        if format!("{:?}", m) == "c3b1" {
            split_perft(board, depth - 1)
        } else if format!("{:?}", m) == "f6h7" {
            split_perft(board, depth - 1)
        } else if format!("{:?}", m) == "f1a6" {
            split_perft(board, depth - 1)
        } else {
            perft(board, depth - 1)
        };
        
        println!("{}: {}", move_str, nodes);
        
        total_nodes += nodes;
        board.unmake_move(&m, &history);
    }
    
    println!("\nTotal: {}", total_nodes);
    total_nodes
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
    
    // kiwipete
    let mut board = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    assert_eq!(perft(&mut board, 1), 48);
    assert_eq!(perft(&mut board, 2), 2039);
    assert_eq!(perft(&mut board, 3), 97862);
    
    // position 3 on chessprogramming.org
    let mut board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
    assert_eq!(perft(&mut board, 1), 14);
    assert_eq!(perft(&mut board, 2), 191);
    assert_eq!(perft(&mut board, 3), 2812);
    assert_eq!(perft(&mut board, 4), 43238);

    // position 4
    let mut board = Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    assert_eq!(perft(&mut board, 1), 6);
    assert_eq!(perft(&mut board, 2), 264);
    assert_eq!(perft(&mut board, 3), 9467);

    // position 5
    let mut board = Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    assert_eq!(perft(&mut board, 1), 44);
    assert_eq!(perft(&mut board, 2), 1486);
    assert_eq!(perft(&mut board, 3), 62379);

    // position 6
    let mut board = Board::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
    assert_eq!(perft(&mut board, 1), 46);
    assert_eq!(perft(&mut board, 2), 2096);
    assert_eq!(perft(&mut board, 3), 89890);
}

#[test]
fn test_split_perft() {
    // startpos
    let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let start = board.hash;

    let result = split_perft(&mut board, 4);
    let result2 = split_perft(&mut board, 4);

    assert_eq!(board.hash, start, "Board hash changed");
    assert_eq!(result, result2, "Inconsistent results");
    assert_eq!(result, 197281);

    // kiwipete
    let mut board = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");

    let result = split_perft(&mut board, 3);
    let result2 = split_perft(&mut board, 3);

    assert_eq!(result, result2, "Inconsistent results");

    assert_eq!(result, 97862);

    // position 3
    let mut board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1");
    let result = split_perft(&mut board, 4);

    assert_eq!(result, 43238);
}