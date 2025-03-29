use std::time::Instant;

use mchess::board::Board;
use mchess::search::Minimax;

#[ignore]
#[test] 
fn search_depth_performance() {
    let mut chess = Minimax::new();
    let mut board = Board::from_fen("2k2r2/1ppp4/pn5q/8/8/8/3B1PPP/1Q4K1 w - - 0 1");

    for depth in 1..=7 {
        let start = Instant::now();

        let a = chess.search(&mut board, depth, f64::NEG_INFINITY, f64::INFINITY, true);

        let duration = start.elapsed();
        println!("Depth {}: {:?}, nodes: {}, best_move {:?}", depth, duration, chess.nodes, a.moves.first());

        chess.nodes = 0;
    }
}

#[ignore]
#[test]
fn iterative_deepening_performance() {
    let mut chess = Minimax::new();
    let mut board = Board::from_fen("2k2r2/1ppp4/pn5q/8/8/8/3B1PPP/1Q4K1 w - - 0 1");

    chess.iterative_deepening(&mut board, 10, 20000);
}