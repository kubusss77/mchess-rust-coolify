use mchess::board::Board;

fn main() {
    let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    println!("{:?}", board);

    let test_board = Board::from_fen("8/8/8/8/8/5p2/4P3/8 w - - 0 1");
    println!("{:?}", test_board);

    println!("{:?}", test_board.pieces[1].borrow_mut().get_legal_moves());
}