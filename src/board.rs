use core::fmt;
use std::{collections::HashMap, i64};

use crate::piece::{Piece, PieceColor, PieceType};
use crate::moves::{Move, MoveType, Pin, Position, Vector};
use crate::pieces::bishop::{get_controlled_squares_bishop, get_legal_moves_bishop, get_pins_bishop};
use crate::pieces::king::{get_controlled_squares_king, get_legal_moves_king};
use crate::pieces::knight::{get_controlled_squares_knight, get_legal_moves_knight};
use crate::pieces::pawn::{get_controlled_squares_pawn, get_legal_moves_pawn};
use crate::pieces::queen::{get_controlled_squares_queen, get_legal_moves_queen, get_pins_queen};
use crate::pieces::rook::{get_controlled_squares_rook, get_legal_moves_rook, get_pins_rook};

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

#[derive(Debug, Clone, PartialEq, Copy)]
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
    pub control_table_lookup: HashMap<usize, Vec<(Position, ControlType)>>,
    pub pin_table: Vec<Vec<Vec<(Position, Position)>>>,
    pub pieces: HashMap<usize, Piece>,
    pub moves: i32,
    pub halfmove_clock: i32,
    pub turn: PieceColor,
    pub castling: Castling,
    pub target_square: Option<Position>,
    pub kings: HashMap<PieceColor, Option<Piece>>,
    pub result_cache: ResultType,
    pub total_moves_cache: HashMap<PieceColor, Vec<Move>>,
    pub moves_cache: HashMap<usize, Vec<Move>>,
    pub move_availability: HashMap<usize, bool>,
    pub check: HashMap<PieceColor, CheckInfo>,
    pub hash_table: Vec<i64>,
    pub hash: i64
}

impl Board {
    pub fn new(moves: Option<i32>, halfmove_clock: Option<i32>, turn: Option<PieceColor>, castling: Option<Castling>, target_square: Option<Position>) -> Self {
        Board {
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
            control_table: vec![vec![vec![]; 8]; 8],
            control_table_lookup: HashMap::new(),
            pin_table: vec![vec![vec![]; 8]; 8],
            kings: HashMap::new(),
            result_cache: ResultType::NotCached,
            total_moves_cache: HashMap::new(),
            moves_cache: HashMap::new(),
            move_availability: HashMap::new(),
            check: HashMap::new(),
            hash_table: vec![],
            hash: i64::MAX
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
                        has_moved: false,
                        _directional: match char.to_ascii_lowercase() {
                            'b' | 'r' | 'q' => true,
                            _ => false
                        }
                    };

                    board.board[i][j] = board.pieces.len() as isize;
                    board.pieces.insert(index, piece.clone());
                    board.control_table_lookup.insert(index, vec![]);

                    if piece.piece_type == PieceType::King {
                        board.kings.insert(piece.color.clone(), Some(piece.clone()));
                    }
                }
                i += 1;
            }
        }

        if !board.kings.contains_key(&PieceColor::White) || !board.kings.contains_key(&PieceColor::Black) {
            panic!("Invalid chess board");
        }

        board.turn = if turn == "b" { PieceColor::Black } else { PieceColor::White };
        board.castling.white = (c.contains("K"), c.contains("Q"));
        board.castling.black = (c.contains("k"), c.contains("q"));
        board.halfmove_clock = halfmoves.parse().unwrap();
        board.moves = moves.parse().unwrap();

        board.check.insert(PieceColor::White, CheckInfo {
            checked: false,
            double_checked: false,
            block_positions: None
        });

        board.check.insert(PieceColor::Black, CheckInfo {
            checked: false,
            double_checked: false,
            block_positions: None
        });
        
        if target_square != "-" {
            board.target_square = Some(Position {
                x: "abcdefgh".find(target_square.chars().next().unwrap()).unwrap(),
                y: target_square[1..].parse().unwrap()
            })
        }

        board.gen_hash();

        board
    }

    pub fn get_piece(&self, piece_index: usize) -> Option<&Piece> {
        self.pieces.get(&piece_index)
    }

    pub fn get_legal_moves(&mut self, piece_index: usize) -> Vec<Move> {
        if let Some(piece) = self.pieces.get_mut(&piece_index).cloned() {
            let legal_moves_cache = self.moves_cache.get_mut(&piece_index);
            if legal_moves_cache.as_ref().is_some() && legal_moves_cache.as_ref().unwrap().len() > 0 || !self.move_availability.get(&piece_index).or(Some(&false)).unwrap() { return legal_moves_cache.unwrap().clone() };
        
            let moves = match piece.piece_type {
                PieceType::Pawn => get_legal_moves_pawn(piece, self),
                PieceType::Bishop => get_legal_moves_bishop(piece, self),
                PieceType::Knight => get_legal_moves_knight(piece, self),
                PieceType::Rook => get_legal_moves_rook(piece, self),
                PieceType::Queen => get_legal_moves_queen(piece, self),
                PieceType::King => get_legal_moves_king(&piece, self)
            };

            self.moves_cache.insert(piece_index, moves.clone());
            self.move_availability.insert(piece_index, moves.len() > 0);

            moves
        } else {
            vec![]
        }
    }

    pub fn get_pins(&mut self, piece_index: usize) -> Vec<Pin> {
        if let Some(piece) = self.pieces.get_mut(&piece_index).cloned() {
            match piece.piece_type {
                PieceType::Bishop => get_pins_bishop(piece, self),
                PieceType::Rook => get_pins_rook(piece, self),
                PieceType::Queen => get_pins_queen(piece, self),
                _ => vec![]
            }
        } else {
            vec![]
        }
    }

    pub fn update_board(&mut self, reset_clock: bool) {
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
        for piece in self.pieces.values_mut() {
            piece.legal_moves_cache.clear();
        }
        self.result_cache = ResultType::NotCached;
        self.total_moves_cache.clear();
        self.hash ^= self.hash_table[12 * 64 + 4];
        self.hash ^= self.hash_table[12 * 64 + 1];
    }

    pub fn make_move(&mut self, m: Move) {
        let piece_index = m.piece_index;

        if m.move_type.contains(&MoveType::Capture) && m.captured.is_some() {
            let captured = m.captured.clone().unwrap();

            self.pieces.remove(&captured.index);
            
            let captured_piece_index = captured.to_piece_index();
            self.hash ^= self.hash_table[captured_piece_index * 64 + captured.pos.y * 8 + captured.pos.x];
        }

        let piece = self.pieces.get_mut(&piece_index).unwrap();
        let pos = piece.pos;

        self.hash ^= self.hash_table[piece_index * 64 + pos.y * 8 + pos.x];
        self.hash ^= self.hash_table[piece_index * 64 + m.to.y * 8 + m.to.x];


        self.board[pos.x][pos.y] = -1;
        self.board[m.to.x][m.to.y] = piece_index as isize;

        piece.pos = Position { x: m.to.x, y: m.to.y };

        self.check_control(piece_index);

        for e in self.control_table[pos.x][pos.y].clone() {
            self.check_control(e.index);
        }

        for e in self.control_table[m.to.x][m.to.y].clone() {
            self.check_control(e.index);
        }

        if m.move_type.contains(&MoveType::Promotion) && m.promote_to.is_some() {
            self.promote_to(piece_index, m.promote_to.unwrap());
        }

        if m.piece_type == PieceType::King && m.move_type.contains(&MoveType::Castling) && m.with.is_some() {
            let rook = m.with.clone().unwrap();
            self.make_move(Move {
                from: rook.pos,
                to: Position { x: if pos.x == 2 { 3 } else { 5 }, y: pos.y },
                move_type: vec![MoveType::Castling],
                captured: None,
                promote_to: None,
                piece_index: rook.index,
                piece_color: rook.color,
                piece_type: rook.piece_type,
                with: None
            });
        }
    }

    pub fn get_controlled_squares(&mut self, piece_index: usize) -> Vec<Control> {
        if let Some(piece) = self.pieces.get_mut(&piece_index).cloned() {
            match piece.piece_type {
                PieceType::Pawn => get_controlled_squares_pawn(piece, self),
                PieceType::Knight => get_controlled_squares_knight(piece, self),
                PieceType::Bishop => get_controlled_squares_bishop(piece, self),
                PieceType::Rook => get_controlled_squares_rook(piece, self),
                PieceType::Queen => get_controlled_squares_queen(piece, self),
                PieceType::King => get_controlled_squares_king(piece, self),
            }
        } else {
            vec![]
        }
    }

    pub fn check_control(&mut self, piece_index: usize) {
        let piece = self.pieces.get(&piece_index).unwrap().clone();
        let controlled_squares = self.get_controlled_squares(piece_index);

        let king = self.get_king(piece.color.opposite()).unwrap();
        let king_pos = king.pos;

        for control in controlled_squares.clone() {
            if king_pos == control.pos {
                let color = piece.color.opposite();
                let check_info = self.check.get_mut(&color).unwrap();
                if check_info.checked {
                    check_info.double_checked = true;
                } else {
                    if piece._directional {
                        let filtered = controlled_squares.iter().filter(|c| c.direction.unwrap() == control.direction.unwrap() && c.direction.unwrap().in_direction(piece.pos, c.pos) && c.direction.unwrap().in_direction(king_pos, c.pos));
                        check_info.block_positions = Some(filtered.map(|c| c.pos).chain([piece.pos]).collect());
                    } else {
                        check_info.block_positions = Some(vec![piece.pos]);
                    }
                }
            }

            self.control_table[control.pos.x][control.pos.y].push(ControlTableEntry {
                index: piece_index,
                control_type: control.control_type,
                color: piece.color,
                obscured: control.obscured,
                is_king: piece.piece_type == PieceType::King
            });
            let table_lookup_entry = self.control_table_lookup.entry(piece_index).or_insert(vec![]);
            table_lookup_entry.push((control.pos, control.control_type));
        }
    }

    pub fn check_control_all(&mut self) {
        let pieces: Vec<usize> = self.pieces.keys().cloned().collect();
        for piece in pieces {
            self.check_control(piece);
        }
    }

    pub fn promote_to(&mut self, piece_index: usize, piece_type: PieceType) {
        let piece = self.pieces.get_mut(&piece_index).unwrap();
        
        self.hash ^= self.hash_table[piece_index * 64 + piece.pos.y * 8 + piece.pos.x];
        
        piece.piece_type = piece_type;

        self.hash ^= self.hash_table[piece_index * 64 + piece.pos.y * 8 + piece.pos.x];

        self.check_control(piece_index);
    }

    pub fn get_king(&mut self, color: PieceColor) -> Option<Piece> {
        if self.kings.contains_key(&color) {
            return self.kings.get(&color).unwrap().clone()
        }
        let king: Option<Piece> = self.pieces.values().find(|p| p.piece_type == PieceType::King && p.color == color).cloned();
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
                    Some(k) => self.get_legal_moves(k.index),
                    None => vec![]
                };
                self.total_moves_cache.insert(color, moves.clone());
                return moves;
            } else if info.checked {
                let moves: Vec<Move> = self.get_block_moves(color).clone().into_iter().chain(match self.get_king(color) {
                    Some(k) => self.get_legal_moves(k.index),
                    None => vec![]
                }).collect();
                self.total_moves_cache.insert(color, moves.clone());
                return moves;
            }
        }

        let indexes: Vec<usize> = self.pieces.values().filter(|p| p.color == color).map(|p| p.index).collect();
        let total: Vec<Move> = indexes.iter().flat_map(|&index| self.get_legal_moves(index)).collect();

        self.total_moves_cache.insert(color, total.clone());

        total
    }

    pub fn get_block_moves(&mut self, color: PieceColor) -> Vec<Move> {
        // TODO: todo!()
        vec![]
    }

    pub fn get_piece_at(&self, rank: usize, file: usize) -> Option<Piece> {
        if !Board::in_bounds(rank, file) { return None; }
        if self.board[file][rank] > -1 {
            Some(self.pieces.get(&(self.board[file][rank] as usize)).unwrap().clone()) 
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

    pub fn get_control_at(&self, rank: usize, file: usize, color: Option<PieceColor>) -> Vec<ControlTableEntry> {
        if let Some(color) = color {
            self.control_table[rank][file].iter().filter(|c| c.color == color).cloned().collect()
        } else {
            self.control_table[rank][file].clone()
        }
    }

    pub fn is_empty(&self, rank: usize, file: usize) -> bool {
        self.board[file][rank] == -1
    }

    pub fn is_pinned(&self, rank: usize, file: usize) -> bool {
        if !Board::in_bounds(rank, file) { return false };
        if self.is_empty(rank, file) { return false };
        let pin = &self.pin_table[rank][file];
        pin.len() > 0
    }

    pub fn gen_hash(&mut self) {
        let mut hash_array = vec![];
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