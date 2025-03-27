use core::f64;
use std::io;

use mchess::{board::Board, evaluation::evaluate, piece::PieceColor, pieces::pawn::get_controlled_squares_pawn, protocol::UciProtocol, search::Minimax};

fn main() -> io::Result<()> {
    // let mut engine = Minimax::new();

    // let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    // println!("{:?}", board);

    // let mut test_board = Board::from_fen("2k2r2/1ppp4/pn5q/8/8/8/3B1PPP/1Q4K1 w - - 0 1");
    // println!("{:?}", test_board);
    // println!("{:?} {:?}", test_board.get_total_legal_moves(None), PieceColor::White);

    // test_board.check_control_all();

    // println!("{:?}", get_controlled_squares_pawn(&test_board.get_piece(3).unwrap().to_partial(), &test_board));

    // println!("{:?}", evaluate(&mut test_board));

    // // let search = engine.search(&mut test_board, 7, f64::NEG_INFINITY, f64::INFINITY, true);
    // let search = engine.iterative_deepening(&mut test_board, 9, 20000);

    // println!("{:?}", search);

    // println!("{:?}", test_board);

    // // engine.debug_move_sequence(&mut test_board, &search.moves, 6);

    // // let mut test_board_2 = Board::from_fen("2k5/1ppp4/pn6/5Q2/8/2B5/3q1rPP/6K1 w - - 2 3");

    // let mut mate_in_one = Board::from_fen("2k1r3/1ppp4/pn5B/8/8/8/1Q3PPP/6K1 b - - 0 1");

    // let search2 = engine.search(&mut mate_in_one, 3, f64::NEG_INFINITY, f64::INFINITY, true);
    // println!("{:?}", search2);
    // println!("{:?}", mate_in_one);

    // engine.debug_move_sequence(&mut mate_in_one, &search2.moves, 3);

    let mut uci = UciProtocol::new();
    uci.run()?;

    Ok(())
}