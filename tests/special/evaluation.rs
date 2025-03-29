use core::f64;

use mchess::{board::{Board, ResultType}, evaluation::{evaluate, evaluate_kings_safety}, search::Minimax};

#[test]
fn test_evaluation() {
    let mut board = Board::from_fen("8/8/7p/3KNN1k/2p4p/8/3P2p1/8 w - - 0 1");
    println!("{:?}", evaluate(&mut board));
    println!("{:?}", board);

    let mut engine = Minimax::new();

    println!("{:?}", engine.search(&mut board, 7, f64::NEG_INFINITY, f64::INFINITY, true));
}

#[test]
fn test_draw() {
    let mut board = Board::from_fen("5k2/8/3b4/8/5B2/8/2K5/8 w - - 0 1");

    assert_eq!(board.get_result(), ResultType::Draw);

    let mut board = Board::from_fen("5k2/8/3b4/8/4B3/8/2K5/8 w - - 0 1");

    assert_ne!(board.get_result(), ResultType::Draw);
}

#[test]
fn evaluate_king_safety() {
    let mut board = Board::from_fen("6k1/5p2/8/7P/3B2P1/PQ6/1PP5/1K3R2 w - - 0 1");

    println!("{:?}", evaluate_kings_safety(&mut board));

    let mut engine = Minimax::new();
    println!("{:?}", engine.search(&mut board, 7, f64::NEG_INFINITY, f64::INFINITY, true));
}