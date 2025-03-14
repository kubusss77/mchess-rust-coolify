use core::fmt;
use std::{collections::HashMap, i64, cell::RefCell, rc::Rc};

use crate::piece::*;
use crate::piece::pawn::*;
use crate::piece::knight::*;
use crate::piece::bishop::*;
use crate::piece::rook::*;
use crate::piece::queen::*;
use crate::piece::king::*;
use crate::moves::*;

use rand::rngs::StdRng;
use rand::{SeedableRng, Rng};

#[derive(Debug, Clone)]
pub enum ResultType {
    WhiteCheckmate,
    BlackCheckmate,
    Stalemate,
    Draw,
    None,
    NotCached,
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
    pub checked: bool,
    pub double_checked: bool,
    pub block_positions: Option<Vec<Position>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ControlType {
    Control,
    Defend,
    Attack,
}

#[derive(Debug, Clone)]
pub struct Control {
    pub pos: Position,
    pub control_type: ControlType,
    pub color: PieceColor,
    pub direction: Option<Vector>,
    pub obscured: bool
}

#[derive(Debug, Clone)]
pub struct ControlTableEntry {
    pub index: usize,
    pub control_type: ControlType,
    pub color: PieceColor,
    pub obscured: bool,
    pub is_king: bool
}

#[derive(Clone)]
pub struct Board {
    pub board: Vec<Vec<isize>>,
    pub control_table: Vec<Vec<Vec<ControlTableEntry>>>,
    pub control_table_lookup: Vec<Vec<(usize, usize, ControlType)>>,
    pub pin_table: Vec<Vec<Vec<(Position, Position)>>>,
    pub pieces: Vec<PieceRef>,
    pub moves: i32,
    pub halfmove_clock: i32,
    pub turn: PieceColor,
    pub castling: Castling,
    pub target_square: Option<Position>,
    pub kings: HashMap<PieceColor, Option<PieceRef>>,
    pub result_cache: ResultType,
    pub total_moves_cache: HashMap<PieceColor, Vec<Move>>,
    pub check: HashMap<PieceColor, CheckInfo>,
    pub hash_table: Vec<i64>,
    pub hash: i64
}

impl Board {
    pub fn new(moves: Option<i32>, halfmove_clock: Option<i32>, turn: Option<PieceColor>, castling: Option<Castling>, target_square: Option<Position>) -> Self {
        Board {
            board: vec![vec![-1; 8]; 9],
            pieces: vec![],
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
            control_table: vec![vec![vec![]; 8]; 8],
            control_table_lookup: vec![vec![]],
            pin_table: vec![vec![vec![]; 8]; 8],
            kings: HashMap::new(),
            result_cache: ResultType::NotCached,
            total_moves_cache: HashMap::new(),
            check: HashMap::new(),
            hash_table: vec![],
            hash: i64::MAX
        }
    }

    pub fn from_fen(fen: &str) -> Self {
        let mut board = Board::new(None, None, None, None, None);
        let parts: Vec<&str> = fen.split_whitespace().collect();
        let position = parts[0];
        let turn = parts[1];
        let c = parts[2];
        let target_square = parts[3];
        let halfmoves = parts[4];
        let moves = parts[5];

        let ranks: Vec<&str> = position.split('/').collect();

        for (i, rank) in ranks.iter().enumerate() {
            let mut j = 0;
            for char in rank.chars().into_iter() {
                if char.is_digit(10) {
                    j += char.to_digit(10).unwrap() as usize - 1;
                } else {
                    let color = if "PNBRQK".contains(char) {
                        PieceColor::White
                    } else {
                        PieceColor::Black
                    };

                    let piece: PieceRef = match char.to_ascii_lowercase() {
                        'p' => Rc::new(RefCell::new(Pawn::new(board.clone(), color, Position { x: i, y: j }, board.pieces.len()))),
                        'n' => Rc::new(RefCell::new(Knight::new(board.clone(), color, Position { x: i, y: j }, board.pieces.len()))),
                        'b' => Rc::new(RefCell::new(Bishop::new(board.clone(), color, Position { x: i, y: j }, board.pieces.len()))),
                        'r' => Rc::new(RefCell::new(Rook::new(board.clone(), color, Position { x: i, y: j }, board.pieces.len()))),
                        'q' => Rc::new(RefCell::new(Queen::new(board.clone(), color, Position { x: i, y: j }, board.pieces.len()))),
                        'k' => Rc::new(RefCell::new(King::new(board.clone(), color, Position { x: i, y: j }, board.pieces.len()))),
                        _ => panic!("Invalid piece type")
                    };

                    board.board[i][j] = board.pieces.len() as isize;
                    board.pieces.push(piece.clone());
                    board.control_table_lookup.push(vec![]);

                    if piece.borrow().get_base().piece_type == PieceType::King {
                        board.kings.insert(piece.borrow().get_base().color.clone(), Some(piece.clone()));
                    }
                }
                j += 1;
            }
        }

        board.turn = if turn == "b" { PieceColor::Black } else { PieceColor::White };
        board.castling.white = (c.contains("K"), c.contains("Q"));
        board.castling.black = (c.contains("k"), c.contains("q"));
        board.halfmove_clock = halfmoves.parse().unwrap();
        board.moves = moves.parse().unwrap();

        if target_square != "-" {
            board.target_square = Some(Position {
                x: "abcdefgh".find(target_square.chars().next().unwrap()).unwrap(),
                y: target_square[1..].parse().unwrap()
            })
        }

        board.gen_hash();

        board
    }

    pub fn move_(&mut self, reset_clock: bool) {
        self.turn = if self.turn == PieceColor::White {
            PieceColor::Black
        } else {
            PieceColor::White
        };
        if !reset_clock {
            self.halfmove_clock += 1
        } else {
            self.halfmove_clock = 0
        }
        if self.turn == PieceColor::White {
            self.moves += 1;
        }
        self.control_table.clear();
        for piece in self.pieces.iter_mut() {
            piece.borrow_mut().get_base_mut().legal_moves_cache.clear();
        }
        self.result_cache = ResultType::NotCached;
        self.total_moves_cache.clear();
        self.hash ^= self.hash_table[12 * 64 + 4];
        self.hash ^= self.hash_table[12 * 64 + 1];
    }

    pub fn get_king(&mut self, color: PieceColor) -> Option<PieceRef> {
        if self.kings.contains_key(&color) {
            return self.kings.get(&color).cloned().unwrap()
        }
        let king: Option<PieceRef> = self.pieces.iter().find(|p| p.borrow().get_base().piece_type == PieceType::King && p.borrow().get_base().color == color).cloned();
        self.kings.insert(color, king.clone());
        king
    }

    pub fn get_attackers_at(&mut self, rank: usize, file: usize, color: PieceColor) -> Vec<ControlTableEntry> {
        self.control_table[rank][file].iter().filter(|c| c.control_type == ControlType::Attack && c.color == color).cloned().collect()
    }

    pub fn get_total_legal_moves(&mut self, _color: Option<PieceColor>) -> Vec<Move> {
        let color = match _color {
            Some(c) => c,
            None => self.turn
        };

        if self.total_moves_cache.get(&color).is_some_and(|t| t.len() > 0) {
            return self.total_moves_cache.get(&color).unwrap().to_vec().clone();
        }

        let check_info = self.check.get(&color);
        if let Some(info) = check_info {
            if info.double_checked {
                let moves: Vec<Move> = match self.get_king(color) {
                    Some(k) => k.borrow_mut().get_legal_moves().clone(),
                    None => vec![]
                };
                self.total_moves_cache.insert(color, moves.clone());
                return moves;
            } else if info.checked {
                let moves: Vec<Move> = self.get_block_moves(color).clone().into_iter().chain(match self.get_king(color) {
                    Some(k) => k.borrow_mut().get_legal_moves().clone(),
                    None => vec![]
                }).collect();
                self.total_moves_cache.insert(color, moves.clone());
                return moves;
            }
        }

        let iterator = self.pieces.iter().filter(|p| p.borrow().get_base().color == color && p.borrow().get_base().piece_type != PieceType::Dead);
        let total: Vec<Move> = iterator.flat_map(|p| p.borrow_mut().get_legal_moves()).collect();

        self.total_moves_cache.insert(color, total.clone());

        total
    }

    pub fn get_block_moves(&mut self, color: PieceColor) -> Vec<Move> {
        // TODO: todo!()
        vec![]
    }

    pub fn get_piece_at(&self, rank: usize, file: usize) -> Option<PieceRef> {
        if !Board::in_bounds(rank, file) { return None; }
        if self.board[rank][file] > -1 {
            Some(self.pieces[self.board[rank][file] as usize].clone()) 
        } else {
            None
        }
    }

    pub fn square_free(&self, rank: usize, file: usize, color: PieceColor) -> bool {
        if !Board::in_bounds(rank, file) { return false; }
        let piece = self.get_piece_at(rank, file);
        if let Some(p) = piece {
            let piece_ = p.borrow();
            piece_.get_base().color != color && piece_.get_base().piece_type != PieceType::King
        } else {
            true
        }
    }

    pub fn in_bounds(rank: usize, file: usize) -> bool {
        rank >= 0 && rank < 8 && file >= 0 && file < 8
    }

    pub fn get_control_at(&self, rank: usize, file: usize, color: Option<PieceColor>) -> Vec<ControlTableEntry> {
        if let Some(color) = color {
            self.control_table[rank][file].iter().filter(|c| c.color == color).cloned().collect()
        } else {
            self.control_table[rank][file].clone()
        }
    }

    pub fn is_empty(&self, rank: usize, file: usize) -> bool {
        self.board[rank][file] == -1
    }

    pub fn is_pinned(&self, rank: usize, file: usize) -> bool {
        if !Board::in_bounds(rank, file) { return false };
        if self.is_empty(rank, file) { return false };
        let pin = &self.pin_table[rank][file];
        pin.len() > 0
    }

    pub fn do_move(&mut self, move_: Move) {
        self.pieces[move_.piece_index].borrow_mut().get_base_mut().move_(move_.clone());
        self.move_(move_.move_type.contains(&MoveType::Capture) || move_.move_type.contains(&MoveType::Promotion));
    }

    pub fn gen_hash(&mut self) {
        let mut hash_array = vec![];
        let mut hash = i64::MAX;

        let mut rng = StdRng::seed_from_u64(9009);

        for _ in 0..((64 * 12) + 4 + 2 + 8) {
            hash_array.push(rng.random::<i64>());
        }

        for piece in &self.pieces {
            let _ref = piece.borrow();
            if _ref.get_base().piece_type == PieceType::Dead { continue; };
            let pos = &_ref.get_base().pos;
            let piece_index = _ref.get_base().to_piece_index();

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
        write!(f, "  ");
        for i in 0..8 {
            write!(f, "{} ", "abcdefgh".chars().nth(i).unwrap());
        }
        write!(f, "\n");
        for row in 0..8 {
            write!(f, "{} ", 8 - row);
            for col in 0..8 {
                let piece = self.board[row][col];
                if piece == -1 {
                    write!(f, ". ");
                } else {
                    let borrow = self.pieces[piece as usize].borrow();
                    let piece_base = borrow.get_base();
                    let piece_color = piece_base.color.clone();
                    let piece_type = piece_base.piece_type.clone();

                    let piece_char = match piece_type {
                        PieceType::Pawn => "p",
                        PieceType::Knight => "n",
                        PieceType::Bishop => "b",
                        PieceType::Rook => "r",
                        PieceType::Queen => "q",
                        PieceType::King => "k",
                        _ => ""
                    };

                    write!(f, "{} ", if piece_color == PieceColor::White {
                        piece_char.to_uppercase()
                    } else {
                        piece_char.to_owned()
                    });
                }
            }
            write!(f, "\n");
        }
        Ok(())
    }
}