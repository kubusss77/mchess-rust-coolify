use mchess::{board::Board, evaluation::evaluate, pieces::pawn::get_controlled_squares_pawn};

fn main() {
    let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    println!("{:?}", board);

    let mut test_board = Board::from_fen("8/k6K/8/8/8/5p2/4P3/8 w - - 0 1");

    test_board.check_control_all();

    println!("{:?}", get_controlled_squares_pawn(test_board.get_piece(3).unwrap().to_owned(), &mut test_board));

    println!("{:?}", evaluate(&mut test_board));
}