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
}

mod common;
mod performance;