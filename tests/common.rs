use mchess::moves::Position;

pub fn alg(s: &str) -> Position {
    let mut chars = s.chars();
    
    let file = chars.next().expect("Algebraic notation too short!");
    let rank = chars.next().expect("Algebraic notation too short!");

    let x = file as u8 - b'a' as u8;
    let y = 8 - (rank as u8 - b'0' as u8);

    Position {
        x: x as usize,
        y: y as usize
    }
}

pub fn show_mask(mask: u64) {
    print!("  ");
    for i in 0..8 {
        print!("{} ", "abcdefgh".chars().nth(i).unwrap());
    }
    print!("\n");
    for rank in 0..8 {
        print!("{} ", 8 - rank);
        for file in 0..8 {
            let pos = Position { x: file, y: rank };
            let q = mask & pos.to_bitboard();
            if q == 0 {
                print!(". ");
            } else {
                print!("1 ");
            }
        }
        print!("\n");
    }
}