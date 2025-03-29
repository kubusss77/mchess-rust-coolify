use mchess::{board::{Board, ControlBitboards}, moves::{Move, MoveType}, piece::PieceColor};

fn control_bitboards_match(a: &ControlBitboards, b: &ControlBitboards) -> bool {
    if a.white_control != b.white_control || a.black_control != b.black_control ||
       a.white_attack != b.white_attack || a.black_attack != b.black_attack ||
       a.white_defend != b.white_defend || a.black_defend != b.black_defend {
        return false;
    }
    
    if a.piece_control.len() != b.piece_control.len() {
        return false;
    }
    
    for (index, control) in &a.piece_control {
        if b.piece_control.get(index) != Some(control) {
            return false;
        }
    }
    
    true
}

#[test]
fn test_move_unmove_control_consistency() {
    let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    
    let initial_control = board.control_bitboards.clone();
    
    let moves = board.get_total_legal_moves(None);
    assert!(!moves.is_empty(), "Expected legal moves");
    
    let m = &moves[0];
    let history = board.make_move(m);
    board.unmake_move(m, &history);
    
    assert!(control_bitboards_match(&board.control_bitboards, &initial_control), "Control bitboards changed after move/unmake sequence");
}


#[test]
fn test_castling_control_consistency() {
    let mut board = Board::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");
    
    let king = board.get_king(PieceColor::White).unwrap();
    let king_moves = board.get_legal_moves(king.index);
    
    let castling_moves: Vec<&Move> = king_moves.iter()
        .filter(|m| m.move_type.contains(&MoveType::Castling))
        .collect();
    
    assert!(!castling_moves.is_empty(), "Expected castling moves");
    
    let initial_control = board.control_bitboards.clone();
    
    let history = board.make_move(castling_moves[0]);
    board.unmake_move(castling_moves[0], &history);
    
    assert!(control_bitboards_match(&board.control_bitboards, &initial_control), "Control bitboards changed after castling move/unmake");
    
    if castling_moves.len() > 1 {
        let history = board.make_move(castling_moves[1]);
        board.unmake_move(castling_moves[1], &history);
        
        assert!(control_bitboards_match(&board.control_bitboards, &initial_control), "Control bitboards changed after other castling move/unmake");
    }
}

#[test]
fn test_ray_piece_control_consistency() {
    let mut board = Board::from_fen("r1bqk2r/ppp2ppp/2n1pn2/3p4/1bPP4/2N1PN2/PP3PPP/R1BQK2R w KQkq - 0 1");
    
    let initial_control = board.control_bitboards.clone();
    
    let moves = board.get_total_legal_moves(None);
    let m = &moves[0];
    
    let history = board.make_move(m);
    board.unmake_move(m, &history);
    
    assert!(control_bitboards_match(&board.control_bitboards, &initial_control), "Control bitboards changed after move with ray pieces");
}

#[test]
fn test_control_validation_by_recalculation() {
    let board = Board::from_fen("r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 1");
    
    let mut temp_board = board.clone();
    
    temp_board.control_bitboards = ControlBitboards { 
        piece_control: std::collections::HashMap::new(),
        white_control: 0u64,
        black_control: 0u64,
        white_attack: 0u64,
        black_attack: 0u64,
        white_defend: 0u64,
        black_defend: 0u64,
        control_entries: std::collections::HashMap::new()
    };
    
    temp_board.check_control_all();
    
    assert!(control_bitboards_match(&board.control_bitboards, &temp_board.control_bitboards), "Current control state doesn't match freshly calculated state");
}

#[test]
fn test_multiple_moves_control_consistency() {
    let mut board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    
    let initial_control = board.control_bitboards.clone();
    
    let mut histories = Vec::new();
    
    for _ in 0..5 {
        let moves = board.get_total_legal_moves(None);
        if moves.is_empty() { break; }
        
        let m = moves[0].clone();
        let history = board.make_move(&m);
        histories.push((m, history));
    }
    
    for (m, history) in histories.into_iter().rev() {
        board.unmake_move(&m, &history);
    }
    
    assert!(control_bitboards_match(&board.control_bitboards, &initial_control), "Control bitboards changed after multiple moves/unmakes");
}