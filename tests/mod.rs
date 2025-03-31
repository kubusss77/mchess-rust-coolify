mod piece {
    pub mod pawn;
    pub mod knight;
    pub mod bishop;
    pub mod rook;
    pub mod queen;
    pub mod king;
}

mod special {
    pub mod pin;
    pub mod perft;
    pub mod make_unmake;
    pub mod control;
    pub mod blocks;
    pub mod evaluation;
    pub mod castling;
    pub mod capture;
    // position-specific tests
    pub mod pos;
}

mod common;
mod performance;