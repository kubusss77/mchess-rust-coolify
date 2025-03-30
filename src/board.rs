use core::fmt;
use std::collections::HashSet;
use std::{collections::HashMap, i64};

use crate::r#const::{MAX_PHASE, MOBILITY_VALUE, MOVE_PREALLOC};
use crate::piece::{BasePiece, PartialPiece, Piece, PieceColor, PieceType};
use crate::moves::{Move, MoveType, Pin, Position, Vector};
use crate::pieces::bishop::{get_controlled_squares_bishop, get_legal_moves_bishop, get_pins_bishop};
use crate::pieces::bitboard::COLOR_MASK;
use crate::pieces::king::{get_controlled_squares_king, get_legal_moves_king};
use crate::pieces::knight::{get_controlled_squares_knight, get_legal_moves_knight};
use crate::pieces::pawn::{get_controlled_squares_pawn, get_legal_moves_pawn};
use crate::pieces::queen::{get_controlled_squares_queen, get_legal_moves_queen, get_pins_queen};
use crate::pieces::rook::{get_controlled_squares_rook, get_legal_moves_rook, get_pins_rook};

use rand::rngs::StdRng;
use rand::{SeedableRng, Rng};

#[derive(Debug, Clone, PartialEq)]
pub enum ResultType {
    WhiteCheckmate,
    BlackCheckmate,
    Stalemate,
    Draw,
    None,
    NotCached,
}

impl ResultType {
    pub fn is_end(&self) -> bool{
        if self == &ResultType::None || self == &ResultType::NotCached {
            false
        } else {
            true
        }
    }
}

#[derive(Debug, Clone)]
pub struct Castling {
    pub white: (bool, bool),
    pub black: (bool, bool),
}

impl Castling {
    pub fn can_castle_ks(&self, color: PieceColor) -> bool {
        match color {
            PieceColor::White => self.white.0,
            PieceColor::Black => self.black.0
        }
    }

    pub fn can_castle_qs(&self, color: PieceColor) -> bool {
        match color {
            PieceColor::White => self.white.1,
            PieceColor::Black => self.black.1
        }
    }
}

#[derive(Debug, Clone)]
pub struct CheckInfo {
    pub checked: u64,
    pub double_checked: u64,
    pub block_positions: Option<Vec<Position>>,
    pub block_mask: u64
}

impl CheckInfo {
    pub fn default() -> CheckInfo {
        CheckInfo { checked: 0u64, double_checked: 0u64, block_positions: None, block_mask: !0u64 }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum ControlType {
    Control,
    Defend,
    Attack,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ControlThreat {
    Threatning,
    PotentialMove,
    All
}

impl ControlThreat {
    pub fn is_attack(&self) -> bool {
        matches!(self, Self::Threatning | Self::All)
    }

    pub fn is_move(&self) -> bool {
        matches!(self, Self::PotentialMove | Self::All)
    }
}

#[derive(Debug, Clone)]
pub struct Control {
    pub pos: Position,
    pub control_type: ControlType,
    pub color: PieceColor,
    pub direction: Option<Vector>,
    pub obscured: bool,
    pub threat: ControlThreat
}

#[derive(Debug, Clone)]
pub struct ControlTableEntry {
    pub index: usize,
    pub control_type: ControlType,
    pub color: PieceColor,
    pub obscured: bool,
    pub is_king: bool,
    pub origin: PartialPiece,
    pub threat: ControlThreat
}

impl ControlTableEntry {
    pub fn to_move(&self, board: &Board, position: Position) -> Move {
        Move {
            from: self.origin.pos,
            to: position,
            piece_index: self.index,
            piece_color: self.color,
            piece_type: self.origin.piece_type,
            move_type: vec![if self.control_type == ControlType::Attack {
                MoveType::Capture
            } else {
                MoveType::Normal
            }],
            captured: board.get_piece_at(position.y, position.x),
            promote_to: None,
            with: None
        }
    }
}

#[derive(Clone)]
pub struct ControlBitboards {
    pub piece_control: HashMap<usize, u64>,

    pub white_control: u64,
    pub black_control: u64,

    pub white_attack: u64,
    pub black_attack: u64,
    pub white_defend: u64,
    pub black_defend: u64,

    pub control_entries: HashMap<u64, Vec<ControlTableEntry>>
}

impl ControlBitboards {
    pub fn clear(&mut self) {
        self.white_control = 0;
        self.black_control = 0;
        self.white_attack = 0;
        self.black_attack = 0;
        self.white_defend = 0;
        self.black_defend = 0;

        self.piece_control.clear();
        self.control_entries.clear();
    }
}

pub type ControlTable = Vec<Vec<Vec<ControlTableEntry>>>;
pub type ControlTableLookup = HashMap<usize, Vec<(Position, ControlType)>>;

#[derive(Clone)]
pub struct MoveInfo {
    pub hash: i64,
    pub captured_piece: Option<Piece>,
    pub halfmove_clock: i32,
    pub white_check: CheckInfo,
    pub black_check: CheckInfo,
    pub turn: PieceColor,
    pub castling: Castling,
    pub promoted_type: Option<PieceType>,
    pub affected_pieces: Vec<usize>,
    pub control_bitboards: ControlBitboards,
    pub target_square: Option<Position>,
    pub target_piece: i32
}

#[derive(Clone)]
pub struct Board {
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
    pub empty_squares: u64,

    pub board: Vec<Vec<isize>>,
    // pub control_table: ControlTable,
    // pub control_table_lookup: ControlTableLookup,
    pub pin_table: Vec<Vec<Vec<Pin>>>,
    pub pieces: HashMap<usize, Piece>,
    pub moves: i32,
    pub halfmove_clock: i32,
    pub turn: PieceColor,
    pub castling: Castling,
    pub target_square: Option<Position>,
    pub target_piece: i32,
    pub result_cache: ResultType,
    pub total_moves_cache: HashMap<PieceColor, Vec<Move>>,
    pub moves_cache: HashMap<usize, Vec<Move>>,
    pub move_availability: HashMap<usize, bool>,
    pub check: HashMap<PieceColor, CheckInfo>,
    pub hash_table: Vec<i64>,
    pub hash: i64,
    pub mobility_cache: HashMap<usize, f64>,

    pub control_bitboards: ControlBitboards
}

impl Board {
    pub fn new(moves: Option<i32>, halfmove_clock: Option<i32>, turn: Option<PieceColor>, castling: Option<Castling>, target_square: Option<Position>) -> Self {
        Board {
            white_pawns: 0,
            white_knights: 0,
            white_bishops: 0,
            white_rooks: 0,
            white_queens: 0,
            white_king: 0,
            white_pieces: 0,

            black_pawns: 0,
            black_knights: 0,
            black_bishops: 0,
            black_rooks: 0,
            black_queens: 0,
            black_king: 0,
            black_pieces: 0,

            all_pieces: 0,
            empty_squares: !0,

            board: vec![vec![-1; 8]; 9],
            pieces: HashMap::new(),
            moves: match moves {
                Some(a) => a,
                None => 1
            },
            halfmove_clock: match halfmove_clock {
                Some(a) => a,
                None => 0
            },
            turn: match turn {
                Some(a) => a,
                None => PieceColor::White
            },
            castling: match castling {
                Some(a) => a,
                None => Castling {
                    white: (true, true),
                    black: (true, true)
                }
            },
            target_square,
            target_piece: -1,
            // control_table: vec![vec![vec![]; 8]; 8],
            // control_table_lookup: HashMap::new(),
            pin_table: vec![vec![vec![]; 8]; 8],
            result_cache: ResultType::NotCached,
            total_moves_cache: HashMap::new(),
            moves_cache: HashMap::new(),
            move_availability: HashMap::new(),
            check: HashMap::new(),
            hash_table: Vec::with_capacity(782),
            hash: i64::MAX,
            mobility_cache: HashMap::new(),

            control_bitboards: ControlBitboards { 
                piece_control: HashMap::new(),
                white_control: 0u64,
                black_control: 0u64,
                white_attack: 0u64,
                black_attack: 0u64,
                white_defend: 0u64,
                black_defend: 0u64,
                control_entries: HashMap::new()
            }
        }
    }

    pub fn from_fen(fen: &str) -> Board {
        let mut board = Board::new(None, None, None, None, None);
        let parts: Vec<&str> = fen.split_whitespace().collect();
        let position = parts[0];
        let turn = parts[1];
        let c = parts[2];
        let target_square = parts[3];
        let halfmoves = parts[4];
        let moves = parts[5];

        let ranks: Vec<&str> = position.split('/').collect();

        for (j, rank) in ranks.iter().enumerate() {
            let mut i = 0;
            for char in rank.chars().into_iter() {
                if char.is_digit(10) {
                    i += char.to_digit(10).unwrap() as usize - 1;
                } else {
                    let color = if "PNBRQK".contains(char) {
                        PieceColor::White
                    } else {
                        PieceColor::Black
                    };

                    let index = board.pieces.len();

                    let piece: Piece = Piece {
                        piece_type: match char.to_ascii_lowercase() {
                            'p' => PieceType::Pawn,
                            'n' => PieceType::Knight,
                            'b' => PieceType::Bishop,
                            'r' => PieceType::Rook,
                            'q' => PieceType::Queen,
                            'k' => PieceType::King,
                            _ => panic!("Invalid piece type")
                        },
                        color,
                        pos: Position { x: i, y: j },
                        index,
                        legal_moves_cache: vec![],
                        legal_moves: true,
                        _directional: match char.to_ascii_lowercase() {
                            'b' | 'r' | 'q' => true,
                            _ => false
                        }
                    };

                    board.board[i][j] = board.pieces.len() as isize;
                    board.pieces.insert(index, piece.clone());
                    // board.control_table_lookup.insert(index, vec![]);

                    board.bb_or_pos(piece.get_base(), piece.pos);
                }
                i += 1;
            }
        }

        if board.white_king == 0u64 || board.black_king == 0u64 {
            panic!("Invalid chess board");
        }

        board.turn = if turn == "b" { PieceColor::Black } else { PieceColor::White };
        board.castling.white = (c.contains("K"), c.contains("Q"));
        board.castling.black = (c.contains("k"), c.contains("q"));
        board.halfmove_clock = halfmoves.parse().unwrap();
        board.moves = moves.parse().unwrap();

        board.check.insert(PieceColor::White, CheckInfo {
            checked: 0u64,
            double_checked: 0u64,
            block_positions: None,
            block_mask: !0u64
        });

        board.check.insert(PieceColor::Black, CheckInfo {
            checked: 0u64,
            double_checked: 0u64,
            block_positions: None,
            block_mask: !0u64
        });
        
        if target_square != "-" {
            board.target_square = Some(Position {
                x: "abcdefgh".find(target_square.chars().next().unwrap()).unwrap(),
                y: 8 - target_square[1..].parse::<usize>().unwrap()
            })
        }

        board.gen_hash();

        board.check_control_all();
        board.update_pins();

        board
    }

    pub fn get_piece_at_bitboard(&self, square: u64) -> Option<BasePiece> {
        if square & self.white_pawns != 0 { return Some((PieceType::Pawn, PieceColor::White)); }
        if square & self.white_knights != 0 { return Some((PieceType::Knight, PieceColor::White)); }
        if square & self.white_bishops != 0 { return Some((PieceType::Bishop, PieceColor::White)); }
        if square & self.white_rooks != 0 { return Some((PieceType::Rook, PieceColor::White)); }
        if square & self.white_queens != 0 { return Some((PieceType::Queen, PieceColor::White)); }
        if square & self.white_king != 0 { return Some((PieceType::King, PieceColor::White)); }
        if square & self.black_pawns != 0 { return Some((PieceType::Pawn, PieceColor::Black)); }
        if square & self.black_knights != 0 { return Some((PieceType::Knight, PieceColor::Black)); }
        if square & self.black_bishops != 0 { return Some((PieceType::Bishop, PieceColor::Black)); }
        if square & self.black_rooks != 0 { return Some((PieceType::Rook, PieceColor::Black)); }
        if square & self.black_queens != 0 { return Some((PieceType::Queen, PieceColor::Black)); }
        if square & self.black_king != 0 { return Some((PieceType::King, PieceColor::Black)); }
        None
    }

    pub fn bb_or_pos(&mut self, piece: BasePiece, pos: Position) {
        let square = pos.to_bitboard();

        match piece {
            (PieceType::Pawn, PieceColor::White) => self.white_pawns |= square,
            (PieceType::Knight, PieceColor::White) => self.white_knights |= square,
            (PieceType::Bishop, PieceColor::White) => self.white_bishops |= square,
            (PieceType::Rook, PieceColor::White) => self.white_rooks |= square,
            (PieceType::Queen, PieceColor::White) => self.white_queens |= square,
            (PieceType::King, PieceColor::White) => self.white_king |= square,
            (PieceType::Pawn, PieceColor::Black) => self.black_pawns |= square,
            (PieceType::Knight, PieceColor::Black) => self.black_knights |= square,
            (PieceType::Bishop, PieceColor::Black) => self.black_bishops |= square,
            (PieceType::Rook, PieceColor::Black) => self.black_rooks |= square,
            (PieceType::Queen, PieceColor::Black) => self.black_queens |= square,
            (PieceType::King, PieceColor::Black) => self.black_king |= square,
        }

        if piece.1 == PieceColor::White {
            self.white_pieces |= square;
        } else {
            self.black_pieces |= square;
        }
        self.all_pieces |= square;
        self.empty_squares = !self.all_pieces;
    }

    pub fn bb_and_rev_pos(&mut self, piece: BasePiece, pos: Position) {
        let square = pos.to_bitboard();
        match piece {
            (PieceType::Pawn, PieceColor::White) => self.white_pawns &= !square,
            (PieceType::Knight, PieceColor::White) => self.white_knights &= !square,
            (PieceType::Bishop, PieceColor::White) => self.white_bishops &= !square,
            (PieceType::Rook, PieceColor::White) => self.white_rooks &= !square,
            (PieceType::Queen, PieceColor::White) => self.white_queens &= !square,
            (PieceType::King, PieceColor::White) => self.white_king &= !square,
            (PieceType::Pawn, PieceColor::Black) => self.black_pawns &= !square,
            (PieceType::Knight, PieceColor::Black) => self.black_knights &= !square,
            (PieceType::Bishop, PieceColor::Black) => self.black_bishops &= !square,
            (PieceType::Rook, PieceColor::Black) => self.black_rooks &= !square,
            (PieceType::Queen, PieceColor::Black) => self.black_queens &= !square,
            (PieceType::King, PieceColor::Black) => self.black_king &= !square,
        }
    }
    
    pub fn update_bitboard_pos(&mut self, piece: BasePiece, from: Position, to: Position) {
        
        self.bb_or_pos(piece, to);
        self.bb_and_rev_pos(piece, from);
        
        let to_bb = to.to_bitboard();
        let from_bb = from.to_bitboard();

        if piece.1 == PieceColor::White {
            self.white_pieces &= !from_bb;
            self.white_pieces |= to_bb;
        } else {
            self.black_pieces &= !from_bb;
            self.black_pieces |= to_bb;
        }
        self.all_pieces &= !from_bb;
        self.all_pieces |= to_bb;
        self.empty_squares = !self.all_pieces;
    }

    pub fn clear(&mut self) {
        self.moves_cache.clear();
        self.total_moves_cache.clear();
        self.check.clear();

        self.check.insert(PieceColor::White, CheckInfo {
            checked: 0u64,
            double_checked: 0u64,
            block_positions: None,
            block_mask: !0u64
        });

        self.check.insert(PieceColor::Black, CheckInfo {
            checked: 0u64,
            double_checked: 0u64,
            block_positions: None,
            block_mask: !0u64
        });
        
        self.result_cache = ResultType::NotCached;

        self.mobility_cache.clear();
    }

    pub fn get_piece(&self, piece_index: usize) -> Option<&Piece> {
        self.pieces.get(&piece_index)
    }

    // TODO: discovered attacks
    // nevermind, i forgot i already had pin detection
    pub fn get_legal_moves(&self, piece_index: usize) -> Vec<Move> {
        if !self.pieces.contains_key(&piece_index) {
            return Vec::with_capacity(0);
        }

        // if let Some(cached_moves) = self.moves_cache.get(&piece_index) {
        //     if !cached_moves.is_empty() ||
        //        !self.move_availability.get(&piece_index).unwrap_or(&false) {
        //         return cached_moves.clone();
        //     }
        // }

        let piece = self.pieces.get(&piece_index).unwrap();
        
        let mut moves = match piece.piece_type {
            PieceType::Pawn => get_legal_moves_pawn(&piece, self),
            PieceType::Knight => get_legal_moves_knight(&piece, self),
            PieceType::Bishop => get_legal_moves_bishop(&piece, self),
            PieceType::Rook => get_legal_moves_rook(&piece, self),
            PieceType::Queen => get_legal_moves_queen(&piece, self),
            PieceType::King => get_legal_moves_king(&piece, self)
        };

        for m in &mut moves {
            if self.would_check(&m) {
                m.move_type.extend([MoveType::Check]);
            }
        }

        // self.moves_cache.insert(piece_index, moves.clone());
        // self.move_availability.insert(piece_index, moves.len() > 0);

        moves
    }

    pub fn get_pins(&self, piece_index: usize) -> Vec<Pin> {
        if let Some(piece) = self.pieces.get(&piece_index) {
            match piece.piece_type {
                PieceType::Bishop => get_pins_bishop(piece, self),
                PieceType::Rook => get_pins_rook(piece, self),
                PieceType::Queen => get_pins_queen(piece, self),
                _ => Vec::with_capacity(0)
            }
        } else {
            Vec::with_capacity(0)
        }
    }

    pub fn update_board(&mut self, reset_clock: bool) {
        self.turn = self.turn.opposite();
        if !reset_clock {
            self.halfmove_clock += 1
        } else {
            self.halfmove_clock = 0
        }
        if self.turn == PieceColor::White {
            self.moves += 1;
        }

        for piece in self.pieces.values_mut() {
            piece.legal_moves_cache.clear();
        }
        self.result_cache = ResultType::NotCached;
        self.total_moves_cache.clear();
        self.moves_cache.clear();
        self.hash ^= self.hash_table[12 * 64 + 4];
        self.hash ^= self.hash_table[12 * 64 + 5];
    }

    pub fn make_move(&mut self, m: &Move) -> MoveInfo {
        if !self.pieces.contains_key(&m.piece_index) { println!("{:?} {}\n{:?}", m, m.piece_index, self); }
        let mut history = MoveInfo {
            hash: self.hash,
            captured_piece: m.captured.clone(),
            halfmove_clock: self.halfmove_clock,
            white_check: self.check.get(&PieceColor::White).unwrap_or(&CheckInfo::default()).clone(),
            black_check: self.check.get(&PieceColor::Black).unwrap_or(&CheckInfo::default()).clone(),
            turn: self.turn,
            castling: self.castling.clone(),
            promoted_type: if m.move_type.contains(&MoveType::Promotion) {
                Some(self.pieces.get(&m.piece_index).unwrap().piece_type)
            } else {
                None
            },
            control_bitboards: self.control_bitboards.clone(),
            affected_pieces: Vec::with_capacity(0),
            target_square: self.target_square.clone(),
            target_piece: self.target_piece
        };

        let piece_index = m.piece_index;

        self.update_bitboard_pos((m.piece_type, m.piece_color), m.from, m.to);

        if m.move_type.contains(&MoveType::Capture) && m.captured.is_some() {
            let captured = m.captured.as_ref().unwrap();

            self.pieces.remove(&captured.index);
            
            let captured_piece_index = captured.to_piece_index();
            self.hash ^= self.hash_table[captured_piece_index * 64 + captured.pos.y * 8 + captured.pos.x];
        }

        let piece = self.pieces.get_mut(&piece_index).unwrap();
        let pos = piece.pos;

        if m.piece_type == PieceType::Pawn && (m.from.y as isize - m.to.y as isize).abs() == 2 {
            let rank = (m.from.y + m.to.y) / 2;
            self.target_square = Some(Position { x: m.to.x, y: rank });
            self.target_piece = piece.index as i32;
        } else {
            self.target_square = None;
            if self.target_piece > -1 {
                self.target_piece = -1;
            }
        }

        let hash_index = Piece::piece_index(m.piece_type, m.piece_color);

        self.hash ^= self.hash_table[hash_index * 64 + pos.y * 8 + pos.x];
        self.hash ^= self.hash_table[hash_index * 64 + m.to.y * 8 + m.to.x];

        self.board[pos.x][pos.y] = -1;
        self.board[m.to.x][m.to.y] = piece_index as isize;

        piece.pos = Position { x: m.to.x, y: m.to.y };

        self.check_control(piece_index);

        let from_bb = pos.to_bitboard();
        let to_bb = m.to.to_bitboard();

        let from_indices: Vec<usize> = if let Some(entries) = self.control_bitboards.control_entries.get(&from_bb) {
            entries.iter().map(|e| e.index).collect()
        } else {
            Vec::new()
        };

        for &index in &from_indices {
            self.check_control(index);
        }

        let to_indices: Vec<usize> = if let Some(entries) = self.control_bitboards.control_entries.get(&to_bb) {
            entries.iter().map(|e| e.index).collect()
        } else {
            Vec::new()
        };

        for &index in &to_indices {
            self.check_control(index);
        }

        if m.move_type.contains(&MoveType::Promotion) && m.promote_to.is_some() {
            self.promote_to(piece_index, m.promote_to.unwrap());
        }

        if m.piece_type == PieceType::King && m.move_type.contains(&MoveType::Castling) && m.with.is_some() {
            let rook = m.with.clone().unwrap();

            let old_rook_pos = rook.pos;
            let new_rook_pos = Position {
                x: if m.to.x == 2 { 3 } else { 5 },
                y: m.from.y
            };

            self.update_bitboard_pos(rook.get_base(), old_rook_pos, new_rook_pos);

            self.board[old_rook_pos.x][old_rook_pos.y] = -1;
            self.board[new_rook_pos.x][new_rook_pos.y] = rook.index as isize;

            if let Some(piece) = self.pieces.get_mut(&rook.index) {
                piece.pos = new_rook_pos.clone();
            }

            let rook_hash_index = rook.to_piece_index();
            self.hash ^= self.hash_table[rook_hash_index * 64 + old_rook_pos.y * 8 + old_rook_pos.x];
            self.hash ^= self.hash_table[rook_hash_index * 64 + new_rook_pos.y * 8 + new_rook_pos.x];
            
            self.check_control(rook.index);

            match m.piece_color {
                PieceColor::White => {
                    self.castling.white = (false, false);
                    self.hash ^= self.hash_table[12 * 64];
                    self.hash ^= self.hash_table[12 * 64 + 1];
                },
                PieceColor::Black => {
                    self.castling.black = (false, false);
                    self.hash ^= self.hash_table[12 * 64 + 2];
                    self.hash ^= self.hash_table[12 * 64 + 3];
                }
            }
        }

        if m.piece_type == PieceType::King {
            match m.piece_color {
                PieceColor::White => {
                    self.castling.white = (false, false);
                    self.hash ^= self.hash_table[12 * 64];
                    self.hash ^= self.hash_table[12 * 64 + 1];
                },
                PieceColor::Black => {
                    self.castling.black = (false, false);
                    self.hash ^= self.hash_table[12 * 64 + 2];
                    self.hash ^= self.hash_table[12 * 64 + 3];
                }
            }
        }

        if m.piece_type == PieceType::Rook {
            match m.piece_color {
                PieceColor::White => {
                    if pos.x == 0 && self.castling.white.1 {
                        self.castling.white.1 = false;
                        self.hash ^= self.hash_table[12 * 64 + 1];
                    } else if pos.x == 7 && self.castling.white.0 {
                        self.castling.white.0 = false;
                        self.hash ^= self.hash_table[12 * 64];
                    }
                },
                PieceColor::Black => {
                    if pos.x == 0 && self.castling.black.1 {
                        self.castling.black.1 = false;
                        self.hash ^= self.hash_table[12 * 64 + 3];
                    } else if pos.x == 7 && self.castling.black.0 {
                        self.castling.black.0 = false;
                        self.hash ^= self.hash_table[12 * 64 + 2];
                    }
                }
            }
        }

        self.update_board(m.move_type.contains(&MoveType::Capture) || m.move_type.contains(&MoveType::Promotion));
        self.update_pins();

        history.control_bitboards = self.control_bitboards.clone();

        let mut affected = Vec::with_capacity(to_indices.len() + from_indices.len());
        affected.extend(to_indices);
        affected.extend(from_indices);

        history.affected_pieces = affected;

        history
    }

    pub fn unmake_move(&mut self, m: &Move, history: &MoveInfo) {
        let current_position = {
            let piece = self.pieces.get(&m.piece_index).unwrap();
            piece.pos.clone()
        };

        self.update_bitboard_pos((m.piece_type, m.piece_color), m.to, m.from);

        self.board[current_position.x][current_position.y] = -1;
        self.board[m.from.x][m.from.y] = m.piece_index as isize;

        if let Some(piece) = self.pieces.get_mut(&m.piece_index) {
            piece.pos = m.from.clone();
            
            if let Some(original_type) = history.promoted_type {
                piece.piece_type = original_type;
            }
        }

        if let Some(captured) = history.captured_piece.clone() {
            self.pieces.insert(captured.index, captured.clone());
            self.board[m.to.x][m.to.y] = captured.index as isize;
            self.bb_or_pos(captured.get_base(), captured.pos);
        }

        if m.move_type.contains(&MoveType::Castling) && m.with.is_some() {
            let rook = m.with.clone().unwrap();
            
            let old_pos = Position {
                x: if m.to.x < m.from.x { 3 } else { 5 },
                y: m.from.y
            };

            let new_pos = Position {
                x: if m.to.x < m.from.x { 0 } else { 7 },
                y: m.from.y
            };

            self.update_bitboard_pos((PieceType::Rook, m.piece_color), old_pos, new_pos);

            self.board[old_pos.x][old_pos.y] = -1;
            self.board[new_pos.x][new_pos.y] = rook.index as isize;

            if let Some(piece) = self.pieces.get_mut(&rook.index) {
                piece.pos = new_pos.clone();
            }
        }

        self.hash = history.hash;
        self.halfmove_clock = history.halfmove_clock;
        self.turn = history.turn;
        self.castling = history.castling.clone();
        self.target_square = history.target_square;

        self.control_bitboards.clear();

        self.check_control_all();

        self.check.clear();
        self.check.insert(PieceColor::White, history.white_check.clone());
        self.check.insert(PieceColor::Black, history.black_check.clone());

        self.update_pins();
    }

    pub fn move_clone(&mut self, m: &Move) -> Board {
        let mut new_board = self.clone();

        new_board.clear();

        new_board.make_move(m);

        new_board.check_control_all();

        new_board
    }

    fn get_piece_control(&self, partial: &PartialPiece) -> Vec<Control> { 
        match partial.piece_type {
            PieceType::Pawn => get_controlled_squares_pawn(partial, self),
            PieceType::Knight => get_controlled_squares_knight(partial, self),
            PieceType::Bishop => get_controlled_squares_bishop(partial, self),
            PieceType::Rook => get_controlled_squares_rook(partial, self),
            PieceType::Queen => get_controlled_squares_queen(partial, self),
            PieceType::King => get_controlled_squares_king(partial, self),
        }
    }

    pub fn get_controlled_squares(&self, piece_index: usize) -> Vec<Control> {
        if let Some(piece) = self.pieces.get(&piece_index) {
            let partial = &piece.to_partial();
            self.get_piece_control(partial)
        } else {
            Vec::with_capacity(0)
        }
    }

    pub fn clear_control(&mut self, piece_index: usize) {
        if let Some(positions) = self.control_bitboards.piece_control.remove(&piece_index) {
            let piece_color = match self.pieces.get(&piece_index) {
                Some(piece) => piece.color,
                None => return
            };

            let mut rem = positions;
            let mut a = 0;
            while rem != 0 {
                a += 1;
                if a > 100 { panic!("While loop has been running for over 100 iterations"); }
                let index = rem.trailing_zeros() as usize;
                let square = 1u64 << index;

                if let Some(entries) = self.control_bitboards.control_entries.get_mut(&square) {
                    entries.retain(|entry| entry.index != piece_index);

                    if entries.is_empty() {
                        self.control_bitboards.control_entries.remove(&square);
                    }
                }

                if piece_color == PieceColor::White {
                    self.control_bitboards.white_control &= !square;
                    self.control_bitboards.white_attack &= !square;
                    self.control_bitboards.white_defend &= !square;
                } else {
                    self.control_bitboards.black_control &= !square;
                    self.control_bitboards.black_attack &= !square;
                    self.control_bitboards.black_defend &= !square;
                }

                rem &= rem - 1;
            }
        }
    }

    pub fn check_control(&mut self, piece_index: usize) {
        if !self.pieces.contains_key(&piece_index) {
            self.clear_control(piece_index);
            return;
        }

        self.clear_control(piece_index);

        let piece = self.pieces.get(&piece_index).unwrap();
        let controlled_squares = self.get_controlled_squares(piece_index);

        let mut lookup_entries = Vec::with_capacity(controlled_squares.len());

        let king_pos = self.get_king_pos(piece.color.opposite());

        let mut count = 0;

        let mut control_bb = 0u64;

        for control in &controlled_squares {
            let pos = control.pos.to_bitboard();
            control_bb |= pos;

            if control.control_type == ControlType::Control {
                count += 1;
            }

            if king_pos == control.pos {
                let color = piece.color.opposite();
                let check_info = self.check.get_mut(&color).unwrap();
                if check_info.checked != 0 && check_info.checked != control.pos.to_bitboard() {
                    check_info.double_checked |= control.pos.to_bitboard();
                } else {
                    if piece._directional {
                        let filtered = controlled_squares.iter().filter(|c| c.direction.unwrap() == control.direction.unwrap() && c.direction.unwrap().in_direction(piece.pos, c.pos) && c.direction.unwrap().in_direction(king_pos, c.pos) && c.pos != king_pos);
                        check_info.block_positions = Some(filtered.map(|c| c.pos).chain([piece.pos]).collect());
                    } else {
                        check_info.block_positions = Some(vec![piece.pos]);
                    }
                    check_info.block_mask = 0u64;
                    for c in check_info.block_positions.as_ref().unwrap() {
                        check_info.block_mask |= c.to_bitboard();
                    }
                    check_info.checked = control.pos.to_bitboard();
                }
            }

            if piece.color == PieceColor::White {
                self.control_bitboards.white_control |= pos;
                if control.control_type == ControlType::Attack {
                    self.control_bitboards.white_attack |= pos;
                } else if control.control_type == ControlType::Defend {
                    self.control_bitboards.white_defend |= pos;
                }
            } else {
                self.control_bitboards.black_control |= pos;
                if control.control_type == ControlType::Attack {
                    self.control_bitboards.black_attack |= pos;
                } else if control.control_type == ControlType::Defend {
                    self.control_bitboards.black_defend |= pos;
                }
            }

            self.control_bitboards.control_entries
                .entry(pos)
                .or_insert(Vec::new())
                .push(ControlTableEntry {
                    index: piece_index,
                    control_type: control.control_type,
                    color: piece.color,
                    obscured: control.obscured,
                    is_king: piece.piece_type == PieceType::King,
                    origin: piece.to_partial(),
                    threat: control.threat
                });

            lookup_entries.push((control.pos, control.control_type));
        }

        if !lookup_entries.is_empty() {
            self.control_bitboards.piece_control.insert(piece_index, control_bb);
        }

        self.mobility_cache.insert(piece_index, count as f64 * MOBILITY_VALUE);
    }

    pub fn check_control_all(&mut self) {
        let pieces: Vec<usize> = self.pieces.keys().cloned().collect();
        for piece in pieces {
            self.check_control(piece);
        }
    }

    pub fn promote_to(&mut self, piece_index: usize, piece_type: PieceType) {
        let piece = self.pieces.get_mut(&piece_index).unwrap();
        
        self.hash ^= self.hash_table[piece.to_piece_index() * 64 + piece.pos.y * 8 + piece.pos.x];
        
        piece.piece_type = piece_type;

        self.hash ^= self.hash_table[piece.to_piece_index() * 64 + piece.pos.y * 8 + piece.pos.x];

        self.check_control(piece_index);
    }

    pub fn get_king_pos(&self, color: PieceColor) -> Position {
        let square = if color == PieceColor::White {
            self.white_king
        } else {
            self.black_king
        };

        Position::from_bitboard(square)
    }

    pub fn get_king(&self, color: PieceColor) -> Option<Piece> {
        let pos = self.get_king_pos(color);

        self.get_piece_at(pos.y, pos.x)
    }

    pub fn get_result(&mut self) -> ResultType {
        let check = self.check.get(&self.turn).expect("Expected check information for both colors").clone();
        let king_index = self.get_king(self.turn).expect("Expected both kings").index;
        if (check.double_checked != 0u64 || (check.checked != 0u64 && self.get_block_moves(self.turn).is_empty())) && self.get_legal_moves(king_index).is_empty() {
            match self.turn {
                PieceColor::White => ResultType::BlackCheckmate,
                PieceColor::Black => ResultType::WhiteCheckmate
            }
        } else {
            let no_material = (self.white_queens | self.white_rooks | self.white_pawns | self.black_queens | self.black_rooks | self.black_pawns).count_ones() == 0;
            let white_no_minor = (self.white_knights | self.white_bishops).count_ones() == 0;
            let black_no_minor = (self.black_knights | self.black_bishops).count_ones() == 0;
            let white_one_bishop = self.white_bishops.count_ones() == 1 && self.white_knights.count_ones() == 0;
            let black_one_bishop = self.black_bishops.count_ones() == 1 && self.black_knights.count_ones() == 0;
            let white_one_knight = self.white_knights.count_ones() == 1 && self.white_bishops.count_ones() == 0;
            let black_one_knight = self.black_knights.count_ones() == 1 && self.black_bishops.count_ones() == 0;
            if self.halfmove_clock > 100 ||
                (no_material && white_no_minor && black_no_minor) ||
                (no_material && white_no_minor && black_one_bishop) ||
                (no_material && black_no_minor && white_one_bishop) ||
                (no_material && white_no_minor && black_one_knight) ||
                (no_material && black_no_minor && white_one_knight) ||
                (no_material && white_one_bishop && black_one_bishop && self.white_bishops & COLOR_MASK == self.black_bishops & COLOR_MASK) {
                ResultType::Draw
            } else {
                ResultType::None
            }
        }
    }

    fn collect_all_legal_moves(&self, color: PieceColor, moves: &mut Vec<Move>) {
        if moves.is_empty() {
            moves.reserve(MOVE_PREALLOC);
        }

        let piece_indices: HashSet<usize> = self.pieces.iter()
            .filter_map(|(&index, piece)| {
                if piece.color == color {
                    Some(index)
                } else {
                    None
                }
            })
            .collect::<HashSet<usize>>();

        for &index in &piece_indices {
            let piece_moves = self.get_legal_moves(index);
            if !piece_moves.is_empty() {
                moves.extend(piece_moves);
            }
        }
    }

    pub fn get_total_legal_moves(&mut self, _color: Option<PieceColor>) -> Vec<Move> {
        let color = _color.unwrap_or(self.turn);

        if let Some(cached) = self.total_moves_cache.get(&color) {
            if !cached.is_empty() {
                return cached.clone();
            }
        }

        let mut result = Vec::with_capacity(MOVE_PREALLOC);

        if let Some(info) = self.check.get(&color) {
            if info.double_checked != 0u64 {
                if let Some(king) = self.get_king(color) {
                    result = self.get_legal_moves(king.index);
                }
            } else if info.checked != 0u64 {
                if let Some(king) = self.get_king(color) {
                    result = self.get_legal_moves(king.index);

                    let block_moves = self.get_block_moves(color);
                    result.reserve(block_moves.len());
                    result.extend(block_moves)
                }
            } else {
                self.collect_all_legal_moves(color, &mut result);
            }
        } else {
            self.collect_all_legal_moves(color, &mut result);
        }

        self.total_moves_cache.insert(color, result.clone());

        result
    }

    pub fn get_block_moves(&self, color: PieceColor) -> Vec<Move> {
        let block_positions = self.check.get(&color).expect("CheckInfo expected").block_positions.clone().unwrap_or(Vec::with_capacity(0));
        let mut moves = vec![];
        for pos in block_positions {
            let control_at = self.get_control_at(pos.y, pos.x, Some(color), false);
            let control = control_at.iter()
                .filter(|c| !c.obscured && !c.is_king);
            moves.extend(control.map(|c| c.to_move(self, pos)))
        }
        moves
    }

    pub fn would_check(&self, m: &Move) -> bool {
        let partial = PartialPiece {
            piece_type: m.piece_type,
            pos: m.to,
            color: m.piece_color
        };
        let controlled_squares = self.get_piece_control(&partial);
        
        let king_pos = self.get_king_pos(m.piece_color.opposite());
        for control in &controlled_squares {
            if control.pos == king_pos {
                return true
            }
        }
        false
    }

    pub fn get_piece_at(&self, rank: usize, file: usize) -> Option<Piece> {
        if !Board::in_bounds(rank, file) { return None; }
        if self.board[file][rank] > -1 {
            self.pieces.get(&(self.board[file][rank] as usize)).cloned()
        } else {
            None
        }
    }

    pub fn square_free(&self, rank: usize, file: usize, color: PieceColor) -> bool {
        if !Board::in_bounds(rank, file) { return false; }
        let piece = self.get_piece_at(rank, file);
        if let Some(p) = piece {
            p.color != color && p.piece_type != PieceType::King
        } else {
            true
        }
    }

    pub fn in_bounds(rank: usize, file: usize) -> bool {
        rank < 8 && file < 8
    }

    pub fn get_control_at(&self, rank: usize, file: usize, color: Option<PieceColor>, attacks: bool) -> Vec<ControlTableEntry> {
        let pos = Position { x: file, y: rank };
        let square = pos.to_bitboard();
        if let Some(entries) = self.control_bitboards.control_entries.get(&square) {
            if let Some(color) = color {
                entries.iter().filter(|c| c.color == color && if attacks { c.threat.is_attack() } else { c.threat.is_move() }).cloned().collect()
            } else {
                entries.iter().filter(|c| if attacks { c.threat.is_attack() } else { c.threat.is_move() }).cloned().collect()
            }
        } else {
            Vec::new()
        }
    }

    pub fn is_empty(&self, rank: usize, file: usize) -> bool {
        self.board[file][rank] == -1
    }

    pub fn is_pinned(&self, rank: usize, file: usize) -> Option<Vector> {
        if !Board::in_bounds(rank, file) { return None };
        if self.is_empty(rank, file) { return None };
        let pos = Position {
            x: file,
            y: rank
        };
        let pins = &self.pin_table[rank][file];
        let mut dir = None;
        for pin in pins {
            if pin.dir.in_direction(pos, pin.position) && !pin.is_phantom {
                dir = Some(pin.dir);
            }
        }
        return dir;
    }

    pub fn is_phantom_pinned(&self, rank: usize, file: usize) -> bool {
        if !Board::in_bounds(rank, file) { return false };
        if self.is_empty(rank, file) { return false };
        let pos = Position {
            x: file,
            y: rank
        };
        let pins = &self.pin_table[rank][file];
        pins.iter().any(|p| p.dir.in_direction(pos, p.position) && p.is_phantom)
    }

    pub fn update_pins(&mut self) {
        self.pin_table = vec![vec![vec![]; 8]; 8];

        for (&index, piece) in &self.pieces {
            if matches!(piece.piece_type, PieceType::Bishop | PieceType::Rook | PieceType::Queen) {
                let pins = self.get_pins(index);
                for pin in pins {
                    self.pin_table[pin.position.y][pin.position.x].push(pin);
                }
            }
        }
    }

    pub fn calculate_phase(&self) -> f64 {
        let mut phase = MAX_PHASE;

        for piece in self.pieces.values() {
            if piece.piece_type == PieceType::King {
                continue;
            }

            phase -= match piece.piece_type {
                PieceType::Knight => 1,
                PieceType::Bishop => 1,
                PieceType::Rook => 2,
                PieceType::Queen => 4,
                _ => 0
            };
        }

        phase = phase.clamp(0,  MAX_PHASE);

        phase as f64 / MAX_PHASE as f64
    }

    pub fn gen_hash(&mut self) {
        let mut hash_array = Vec::with_capacity(782);
        let mut hash = i64::MAX;

        let mut rng = StdRng::seed_from_u64(9009);

        for _ in 0..((64 * 12) + 4 + 2 + 8) {
            hash_array.push(rng.random::<i64>());
        }

        for piece in self.pieces.values() {
            let pos = piece.pos;
            let piece_index = piece.to_piece_index();

            hash ^= hash_array[piece_index * 64 + pos.y * 8 + pos.x];
        }

        if self.castling.white.0 { hash ^= hash_array[12 * 64]; }
        if self.castling.white.1 { hash ^= hash_array[12 * 64 + 1]; }
        if self.castling.black.0 { hash ^= hash_array[12 * 64 + 2]; }
        if self.castling.black.1 { hash ^= hash_array[12 * 64 + 3]; }

        if self.turn == PieceColor::White {
            hash ^= hash_array[12 * 64 + 4];
        } else {
            hash ^= hash_array[12 * 64 + 5];
        }

        if let Some(t) = &self.target_square {
            hash ^= hash_array[12 * 64 + 4 + 2 + t.y];
        }

        self.hash = hash;
        self.hash_table = hash_array;
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "  ")?;
        for i in 0..8 {
            write!(f, "{} ", "abcdefgh".chars().nth(i).unwrap())?;
        }
        write!(f, "\n")?;
        for rank in 0..8 {
            write!(f, "{} ", 8 - rank)?;
            for file in 0..8 {
                let piece = self.board[file][rank];
                if piece == -1 {
                    write!(f, ". ")?;
                } else {
                    let piece = self.pieces.get(&(piece as usize)).unwrap();

                    let piece_char = match piece.piece_type {
                        PieceType::Pawn => "p",
                        PieceType::Knight => "n",
                        PieceType::Bishop => "b",
                        PieceType::Rook => "r",
                        PieceType::Queen => "q",
                        PieceType::King => "k"
                    };

                    write!(f, "{} ", if piece.color == PieceColor::White {
                        piece_char.to_uppercase()
                    } else {
                        piece_char.to_owned()
                    })?;
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

#[test]
fn result_check() {
    let mut black_checkmate = Board::from_fen("2k5/1ppp4/pn5B/8/8/8/1Q3PPP/4r1K1 w - - 0 1");

    assert!(black_checkmate.get_result() == ResultType::BlackCheckmate);
}

#[test]
fn turn_check() {
    let mut board = Board::from_fen("2k2r2/1ppp4/pn5q/8/8/8/3B1PPP/1Q4K1 w - - 0 1");

    board.update_board(false);

    assert!(board.turn == PieceColor::Black);
}