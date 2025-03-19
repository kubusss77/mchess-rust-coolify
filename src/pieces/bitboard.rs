pub struct Bitboard {
    pub white_pawns: u64,
    pub white_knights: u64,
    pub white_bishops: u64,
    pub white_rooks: u64,
    pub white_queens: u64,
    pub white_king: u64,
    pub black_pawns: u64,
    pub black_knights: u64,
    pub black_bishops: u64,
    pub black_rooks: u64,
    pub black_queens: u64,
    pub black_king: u64,

    pub white_pieces: u64,
    pub black_pieces: u64,
    pub all_pieces: u64,
}

pub fn bit(square: u8) -> u64 {
    1u64 << square
}

pub fn square_to_index(file: u8, rank: u8) -> u8 {
    rank * 8 + file
}

pub fn index_to_square(index: u8) -> (u8, u8) {
    (index % 8, index / 8)
}