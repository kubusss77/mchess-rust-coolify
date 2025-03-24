use std::time::Instant;

use mchess::board::Board;
use mchess::search::Chess;

#[test] 
fn search_depth_performance() {
    let mut chess = Chess::new();
    let mut board = Board::from_fen("2k2r2/1ppp4/pn5q/8/8/8/3B1PPP/1Q4K1 w - - 0 1");

    for depth in 1..=7 {
        let start = Instant::now();

        chess.search(&mut board, depth, f64::NEG_INFINITY, f64::INFINITY, true);

        let duration = start.elapsed();
        println!("Depth {}: {:?}, nodes: {}", depth, duration, chess.nodes);

        chess.nodes = 0;
    }
}