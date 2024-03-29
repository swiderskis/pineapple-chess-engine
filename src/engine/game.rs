use super::{
    attack_tables::AttackTables,
    moves::{Move, MoveType},
    search::Value,
    zobrist_hashes::{self, ZobristKey},
};
use crate::uci::{FenError, InputError};
use num_derive::FromPrimitive;
use num_traits::{AsPrimitive, FromPrimitive, Unsigned};
use std::{
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, Shl, Shr, ShrAssign},
    str::FromStr,
};
use strum::{IntoEnumIterator, ParseError};
use strum_macros::{Display, EnumIter, EnumString};

const HALFMOVE_CLOCK_MAX: u8 = 99;

#[derive(Clone)]
pub struct Game {
    white_pawns: Bitboard,
    white_knights: Bitboard,
    white_bishops: Bitboard,
    white_rooks: Bitboard,
    white_queens: Bitboard,
    white_king: Bitboard,
    black_pawns: Bitboard,
    black_knights: Bitboard,
    black_bishops: Bitboard,
    black_rooks: Bitboard,
    black_queens: Bitboard,
    black_king: Bitboard,
    side_to_move: Side,
    castling_rights: CastlingRights,
    en_passant_square: Option<Square>,
    halfmove_clock: u8,
    zobrist_key: ZobristKey,
}

impl Game {
    pub fn initialise() -> Self {
        Self {
            white_pawns: Bitboard(0),
            white_knights: Bitboard(0),
            white_bishops: Bitboard(0),
            white_rooks: Bitboard(0),
            white_queens: Bitboard(0),
            white_king: Bitboard(0),
            black_pawns: Bitboard(0),
            black_knights: Bitboard(0),
            black_bishops: Bitboard(0),
            black_rooks: Bitboard(0),
            black_queens: Bitboard(0),
            black_king: Bitboard(0),
            side_to_move: Side::White,
            castling_rights: CastlingRights(0),
            en_passant_square: None,
            halfmove_clock: 0,
            zobrist_key: 0,
        }
    }

    pub fn load_fen(&mut self, fen: &[&str]) -> Result<(), InputError> {
        if fen[0] == "startpos" {
            self.white_pawns = Bitboard(0xFF_0000_0000_0000);
            self.white_knights = Bitboard(0x4200_0000_0000_0000);
            self.white_bishops = Bitboard(0x2400_0000_0000_0000);
            self.white_rooks = Bitboard(0x8100_0000_0000_0000);
            self.white_queens = Bitboard(0x800_0000_0000_0000);
            self.white_king = Bitboard(0x1000_0000_0000_0000);

            self.black_pawns = Bitboard(0xFF00);
            self.black_knights = Bitboard(0x42);
            self.black_bishops = Bitboard(0x24);
            self.black_rooks = Bitboard(0x81);
            self.black_queens = Bitboard(0x8);
            self.black_king = Bitboard(0x10);

            self.side_to_move = Side::White;
            self.castling_rights = CastlingRights::initialise("KQkq")?;
            self.en_passant_square = None;
            self.halfmove_clock = 0;

            self.zobrist_key = zobrist_hashes::ZOBRIST_HASHES.generate_key(self);

            return Ok(());
        }

        let mut white_pawns = Bitboard(0);
        let mut white_knights = Bitboard(0);
        let mut white_bishops = Bitboard(0);
        let mut white_rooks = Bitboard(0);
        let mut white_queens = Bitboard(0);
        let mut white_king = Bitboard(0);

        let mut black_pawns = Bitboard(0);
        let mut black_knights = Bitboard(0);
        let mut black_bishops = Bitboard(0);
        let mut black_rooks = Bitboard(0);
        let mut black_queens = Bitboard(0);
        let mut black_king = Bitboard(0);

        let mut square_index = 0;

        for character in fen[0].chars() {
            match character {
                'P' => {
                    white_pawns.set_bit(Square::from_usize(square_index).unwrap());
                    square_index += 1;
                }
                'N' => {
                    white_knights.set_bit(Square::from_usize(square_index).unwrap());
                    square_index += 1;
                }
                'B' => {
                    white_bishops.set_bit(Square::from_usize(square_index).unwrap());
                    square_index += 1;
                }
                'R' => {
                    white_rooks.set_bit(Square::from_usize(square_index).unwrap());
                    square_index += 1;
                }
                'Q' => {
                    white_queens.set_bit(Square::from_usize(square_index).unwrap());
                    square_index += 1;
                }
                'K' => {
                    white_king.set_bit(Square::from_usize(square_index).unwrap());
                    square_index += 1;
                }
                'p' => {
                    black_pawns.set_bit(Square::from_usize(square_index).unwrap());
                    square_index += 1;
                }
                'n' => {
                    black_knights.set_bit(Square::from_usize(square_index).unwrap());
                    square_index += 1;
                }
                'b' => {
                    black_bishops.set_bit(Square::from_usize(square_index).unwrap());
                    square_index += 1;
                }
                'r' => {
                    black_rooks.set_bit(Square::from_usize(square_index).unwrap());
                    square_index += 1;
                }
                'q' => {
                    black_queens.set_bit(Square::from_usize(square_index).unwrap());
                    square_index += 1;
                }
                'k' => {
                    black_king.set_bit(Square::from_usize(square_index).unwrap());
                    square_index += 1;
                }
                '0'..='9' => square_index += character as usize - '0' as usize,
                '/' => {}
                _ => return Err(InputError::InvalidFen(FenError::BoardPosition)),
            }
        }

        let side_to_move = match fen[1] {
            "w" => Side::White,
            "b" => Side::Black,
            _ => return Err(InputError::InvalidFen(FenError::SideToMove)),
        };
        let castling_rights = CastlingRights::initialise(fen[2])?;
        let en_passant_square = Self::parse_en_passant_square(fen[3])?;
        let halfmove_clock = match fen[4].parse() {
            Ok(halfmove_clock) => {
                if halfmove_clock > HALFMOVE_CLOCK_MAX {
                    return Err(InputError::InvalidFen(FenError::InvalidHalfmoveClock));
                }

                halfmove_clock
            }
            Err(_) => return Err(InputError::InvalidFen(FenError::ParseHalfmoveClock)),
        };

        self.white_pawns = white_pawns;
        self.white_knights = white_knights;
        self.white_bishops = white_bishops;
        self.white_rooks = white_rooks;
        self.white_queens = white_queens;
        self.white_king = white_king;

        self.black_pawns = black_pawns;
        self.black_knights = black_knights;
        self.black_bishops = black_bishops;
        self.black_rooks = black_rooks;
        self.black_queens = black_queens;
        self.black_king = black_king;

        self.side_to_move = side_to_move;
        self.castling_rights = castling_rights;
        self.en_passant_square = en_passant_square;
        self.halfmove_clock = halfmove_clock;

        self.zobrist_key = zobrist_hashes::ZOBRIST_HASHES.generate_key(self);

        Ok(())
    }

    pub fn make_move(&mut self, mv: &Move, attack_tables: &AttackTables) -> Result<(), InputError> {
        let mut game_clone = self.clone();
        let side = game_clone.side_to_move;
        let opponent_side = side.opponent_side();
        game_clone
            .mut_piece_bitboard(mv.piece(), side)
            .pop_bit(mv.source_square());
        game_clone.zobrist_key ^=
            zobrist_hashes::ZOBRIST_HASHES.piece_square_hash(mv.piece(), side, mv.source_square());

        if let Some(en_passant_square) = game_clone.en_passant_square {
            game_clone.en_passant_square = None;
            game_clone.zobrist_key ^=
                zobrist_hashes::ZOBRIST_HASHES.en_passant_square_hash(en_passant_square);
        }

        let target_square_index = mv.target_square() as usize;

        match mv.move_type() {
            MoveType::Quiet => {}
            MoveType::Capture => {
                if let Some((piece, side)) = game_clone.piece_at_square(mv.target_square()) {
                    game_clone
                        .mut_piece_bitboard(piece, side)
                        .pop_bit(mv.target_square());
                    game_clone.zobrist_key ^= zobrist_hashes::ZOBRIST_HASHES.piece_square_hash(
                        piece,
                        side,
                        mv.target_square(),
                    );
                }
            }
            MoveType::DoublePawnPush => {
                let en_passant_square = match side {
                    Side::White => Square::from_usize(target_square_index + 8),
                    Side::Black => Square::from_usize(target_square_index - 8),
                }
                .unwrap();
                game_clone.en_passant_square = Some(en_passant_square);
                game_clone.zobrist_key ^=
                    zobrist_hashes::ZOBRIST_HASHES.en_passant_square_hash(en_passant_square);
            }
            MoveType::EnPassant => {
                let capture_square = match side {
                    Side::White => Square::from_usize(target_square_index + 8),
                    Side::Black => Square::from_usize(target_square_index - 8),
                }
                .unwrap();
                game_clone
                    .mut_piece_bitboard(Piece::Pawn, opponent_side)
                    .pop_bit(capture_square);
                game_clone.zobrist_key ^= zobrist_hashes::ZOBRIST_HASHES.piece_square_hash(
                    Piece::Pawn,
                    opponent_side,
                    capture_square,
                );
            }
            MoveType::Castling => match side {
                Side::White => {
                    match mv.target_square() {
                        Square::C1 => {
                            game_clone
                                .mut_piece_bitboard(Piece::Rook, side)
                                .pop_bit(Square::A1);
                            game_clone
                                .mut_piece_bitboard(Piece::Rook, side)
                                .set_bit(Square::D1);
                            game_clone.zobrist_key ^= zobrist_hashes::ZOBRIST_HASHES
                                .piece_square_hash(Piece::Rook, side, Square::A1);
                            game_clone.zobrist_key ^= zobrist_hashes::ZOBRIST_HASHES
                                .piece_square_hash(Piece::Rook, side, Square::D1);
                        }
                        Square::G1 => {
                            game_clone
                                .mut_piece_bitboard(Piece::Rook, side)
                                .pop_bit(Square::H1);
                            game_clone
                                .mut_piece_bitboard(Piece::Rook, side)
                                .set_bit(Square::F1);
                            game_clone.zobrist_key ^= zobrist_hashes::ZOBRIST_HASHES
                                .piece_square_hash(Piece::Rook, side, Square::H1);
                            game_clone.zobrist_key ^= zobrist_hashes::ZOBRIST_HASHES
                                .piece_square_hash(Piece::Rook, side, Square::F1);
                        }
                        _ => {}
                    };
                }
                Side::Black => {
                    match mv.target_square() {
                        Square::C8 => {
                            game_clone
                                .mut_piece_bitboard(Piece::Rook, side)
                                .pop_bit(Square::A8);
                            game_clone
                                .mut_piece_bitboard(Piece::Rook, side)
                                .set_bit(Square::D8);
                            game_clone.zobrist_key ^= zobrist_hashes::ZOBRIST_HASHES
                                .piece_square_hash(Piece::Rook, side, Square::A8);
                            game_clone.zobrist_key ^= zobrist_hashes::ZOBRIST_HASHES
                                .piece_square_hash(Piece::Rook, side, Square::D8);
                        }
                        Square::G8 => {
                            game_clone
                                .mut_piece_bitboard(Piece::Rook, side)
                                .pop_bit(Square::H8);
                            game_clone
                                .mut_piece_bitboard(Piece::Rook, side)
                                .set_bit(Square::F8);
                            game_clone.zobrist_key ^= zobrist_hashes::ZOBRIST_HASHES
                                .piece_square_hash(Piece::Rook, side, Square::H8);
                            game_clone.zobrist_key ^= zobrist_hashes::ZOBRIST_HASHES
                                .piece_square_hash(Piece::Rook, side, Square::F8);
                        }
                        _ => {}
                    };
                }
            },
        }

        match mv.promoted_piece() {
            Some(promoted_piece) => {
                game_clone
                    .mut_piece_bitboard(promoted_piece, side)
                    .set_bit(mv.target_square());
                game_clone.zobrist_key ^= zobrist_hashes::ZOBRIST_HASHES.piece_square_hash(
                    promoted_piece,
                    side,
                    mv.target_square(),
                );
            }
            None => {
                game_clone
                    .mut_piece_bitboard(mv.piece(), side)
                    .set_bit(mv.target_square());
                game_clone.zobrist_key ^= zobrist_hashes::ZOBRIST_HASHES.piece_square_hash(
                    mv.piece(),
                    side,
                    mv.target_square(),
                );
            }
        }

        let king_square = game_clone
            .piece_bitboard(Piece::King, side)
            .get_lsb_square();

        if let Some(king_square) = king_square {
            let own_king_in_check =
                game_clone.is_square_attacked(attack_tables, opponent_side, king_square);

            if own_king_in_check {
                return Err(InputError::IllegalMove);
            }
        }

        game_clone.update_castling_rights(mv);
        game_clone.zobrist_key ^= zobrist_hashes::ZOBRIST_HASHES.side_hash();
        game_clone.side_to_move = opponent_side;
        *self = game_clone;

        Ok(())
    }

    pub fn make_null_move(&mut self) {
        if let Some(square) = self.en_passant_square {
            self.zobrist_key ^= zobrist_hashes::ZOBRIST_HASHES.en_passant_square_hash(square);
        }

        self.zobrist_key ^= zobrist_hashes::ZOBRIST_HASHES.side_hash();
        self.side_to_move = self.side_to_move.opponent_side();
        self.en_passant_square = None;
    }

    pub fn is_square_attacked(
        &self,
        attack_tables: &AttackTables,
        attacking_side: Side,
        square: Square,
    ) -> bool {
        for piece in Piece::iter() {
            let piece_attacks_square = attack_tables.attack_table(
                self.board(None),
                piece,
                attacking_side.opponent_side(),
                square,
            ) & self.piece_bitboard(piece, attacking_side)
                != 0u64;

            if piece_attacks_square {
                return true;
            }
        }

        false
    }

    pub fn board(&self, side: Option<Side>) -> Bitboard {
        match side {
            Some(side) => match side {
                Side::White => {
                    self.white_pawns
                        | self.white_knights
                        | self.white_bishops
                        | self.white_rooks
                        | self.white_queens
                        | self.white_king
                }
                Side::Black => {
                    self.black_pawns
                        | self.black_knights
                        | self.black_bishops
                        | self.black_rooks
                        | self.black_queens
                        | self.black_king
                }
            },
            None => {
                self.white_pawns
                    | self.white_knights
                    | self.white_bishops
                    | self.white_rooks
                    | self.white_queens
                    | self.white_king
                    | self.black_pawns
                    | self.black_knights
                    | self.black_bishops
                    | self.black_rooks
                    | self.black_queens
                    | self.black_king
            }
        }
    }

    pub fn is_square_occupied(&self, square: Square) -> bool {
        self.board(None).bit_occupied(square)
    }

    pub fn side_to_move(&self) -> Side {
        self.side_to_move
    }

    pub fn en_passant_square(&self) -> Option<Square> {
        self.en_passant_square
    }

    pub fn castling_rights_value(&self) -> u8 {
        self.castling_rights.0
    }

    pub fn castling_type_allowed(&self, castling_type: CastlingType) -> bool {
        self.castling_rights.0 & castling_type as u8 != 0
    }

    pub fn piece_bitboard(&self, piece: Piece, side: Side) -> Bitboard {
        match side {
            Side::White => match piece {
                Piece::Pawn => self.white_pawns,
                Piece::Knight => self.white_knights,
                Piece::Bishop => self.white_bishops,
                Piece::Rook => self.white_rooks,
                Piece::Queen => self.white_queens,
                Piece::King => self.white_king,
            },
            Side::Black => match piece {
                Piece::Pawn => self.black_pawns,
                Piece::Knight => self.black_knights,
                Piece::Bishop => self.black_bishops,
                Piece::Rook => self.black_rooks,
                Piece::Queen => self.black_queens,
                Piece::King => self.black_king,
            },
        }
    }

    fn mut_piece_bitboard(&mut self, piece: Piece, side: Side) -> &mut Bitboard {
        match side {
            Side::White => match piece {
                Piece::Pawn => &mut self.white_pawns,
                Piece::Knight => &mut self.white_knights,
                Piece::Bishop => &mut self.white_bishops,
                Piece::Rook => &mut self.white_rooks,
                Piece::Queen => &mut self.white_queens,
                Piece::King => &mut self.white_king,
            },
            Side::Black => match piece {
                Piece::Pawn => &mut self.black_pawns,
                Piece::Knight => &mut self.black_knights,
                Piece::Bishop => &mut self.black_bishops,
                Piece::Rook => &mut self.black_rooks,
                Piece::Queen => &mut self.black_queens,
                Piece::King => &mut self.black_king,
            },
        }
    }

    fn parse_en_passant_square(
        en_passant_square_string: &str,
    ) -> Result<Option<Square>, InputError> {
        if en_passant_square_string == "-" {
            return Ok(None);
        }

        match Square::from_str(en_passant_square_string.to_uppercase().as_str()) {
            Ok(square) => Ok(Some(square)),
            Err(_) => Err(InputError::InvalidFen(FenError::EnPassantSquare)),
        }
    }

    fn update_castling_rights(&mut self, mv: &Move) {
        if self.castling_rights == 0u8 {
            return;
        }

        match self.side_to_move {
            Side::White => {
                if (mv.source_square() == Square::A1 || mv.source_square() == Square::E1)
                    && self.castling_type_allowed(CastlingType::WhiteLong)
                {
                    self.zobrist_key ^=
                        zobrist_hashes::ZOBRIST_HASHES.castling_hash(self.castling_rights.0);
                    self.castling_rights
                        .remove_castling_type(CastlingType::WhiteLong);
                    self.zobrist_key ^=
                        zobrist_hashes::ZOBRIST_HASHES.castling_hash(self.castling_rights.0);
                }

                if (mv.source_square() == Square::E1 || mv.source_square() == Square::H1)
                    && self.castling_type_allowed(CastlingType::WhiteShort)
                {
                    self.zobrist_key ^=
                        zobrist_hashes::ZOBRIST_HASHES.castling_hash(self.castling_rights.0);
                    self.castling_rights
                        .remove_castling_type(CastlingType::WhiteShort);
                    self.zobrist_key ^=
                        zobrist_hashes::ZOBRIST_HASHES.castling_hash(self.castling_rights.0);
                }
            }
            Side::Black => {
                if (mv.source_square() == Square::A8 || mv.source_square() == Square::E8)
                    && self.castling_type_allowed(CastlingType::BlackLong)
                {
                    self.zobrist_key ^=
                        zobrist_hashes::ZOBRIST_HASHES.castling_hash(self.castling_rights.0);
                    self.castling_rights
                        .remove_castling_type(CastlingType::BlackLong);
                    self.zobrist_key ^=
                        zobrist_hashes::ZOBRIST_HASHES.castling_hash(self.castling_rights.0);
                }

                if (mv.source_square() == Square::E8 || mv.source_square() == Square::H8)
                    && self.castling_type_allowed(CastlingType::BlackShort)
                {
                    self.zobrist_key ^=
                        zobrist_hashes::ZOBRIST_HASHES.castling_hash(self.castling_rights.0);
                    self.castling_rights
                        .remove_castling_type(CastlingType::BlackShort);
                    self.zobrist_key ^=
                        zobrist_hashes::ZOBRIST_HASHES.castling_hash(self.castling_rights.0);
                }
            }
        }

        match self.side_to_move {
            Side::White => {
                if mv.target_square() == Square::A8
                    && self.castling_type_allowed(CastlingType::BlackLong)
                {
                    self.zobrist_key ^=
                        zobrist_hashes::ZOBRIST_HASHES.castling_hash(self.castling_rights.0);
                    self.castling_rights
                        .remove_castling_type(CastlingType::BlackLong);
                    self.zobrist_key ^=
                        zobrist_hashes::ZOBRIST_HASHES.castling_hash(self.castling_rights.0);
                }

                if mv.target_square() == Square::H8
                    && self.castling_type_allowed(CastlingType::BlackShort)
                {
                    self.zobrist_key ^=
                        zobrist_hashes::ZOBRIST_HASHES.castling_hash(self.castling_rights.0);
                    self.castling_rights
                        .remove_castling_type(CastlingType::BlackShort);
                    self.zobrist_key ^=
                        zobrist_hashes::ZOBRIST_HASHES.castling_hash(self.castling_rights.0);
                }
            }
            Side::Black => {
                if mv.target_square() == Square::A1
                    && self.castling_type_allowed(CastlingType::WhiteLong)
                {
                    self.zobrist_key ^=
                        zobrist_hashes::ZOBRIST_HASHES.castling_hash(self.castling_rights.0);
                    self.castling_rights
                        .remove_castling_type(CastlingType::WhiteLong);
                    self.zobrist_key ^=
                        zobrist_hashes::ZOBRIST_HASHES.castling_hash(self.castling_rights.0);
                }

                if mv.target_square() == Square::H1
                    && self.castling_type_allowed(CastlingType::WhiteShort)
                {
                    self.zobrist_key ^=
                        zobrist_hashes::ZOBRIST_HASHES.castling_hash(self.castling_rights.0);
                    self.castling_rights
                        .remove_castling_type(CastlingType::WhiteShort);
                    self.zobrist_key ^=
                        zobrist_hashes::ZOBRIST_HASHES.castling_hash(self.castling_rights.0);
                }
            }
        }
    }

    pub fn piece_bitboards(&self) -> [(Bitboard, Piece, Side); 12] {
        [
            (self.white_pawns, Piece::Pawn, Side::White),
            (self.white_knights, Piece::Knight, Side::White),
            (self.white_bishops, Piece::Bishop, Side::White),
            (self.white_rooks, Piece::Rook, Side::White),
            (self.white_queens, Piece::Queen, Side::White),
            (self.white_king, Piece::King, Side::White),
            (self.black_pawns, Piece::Pawn, Side::Black),
            (self.black_knights, Piece::Knight, Side::Black),
            (self.black_bishops, Piece::Bishop, Side::Black),
            (self.black_rooks, Piece::Rook, Side::Black),
            (self.black_queens, Piece::Queen, Side::Black),
            (self.black_king, Piece::King, Side::Black),
        ]
    }

    pub fn piece_at_square(&self, square: Square) -> Option<(Piece, Side)> {
        for (bitboard, piece, side) in self.piece_bitboards() {
            if bitboard.bit_occupied(square) {
                return Some((piece, side));
            }
        }

        None
    }

    pub fn _print(&self) {
        for square in Square::iter() {
            if square.file() == 0 {
                print!("{:<4}", (64 - square as usize) / 8);
            }

            match self.piece_at_square(square) {
                Some((piece, side)) => print!("{:<2}", piece._to_char(Some(side))),
                None => print!(". "),
            }

            if square.file() == 7 {
                println!();
            }
        }

        println!();
        println!("    a b c d e f g h");
        println!();
        println!("Side to move: {:?}", self.side_to_move);
        println!("En passant square: {:?}", self.en_passant_square);
        println!("Castling rights: {}", self.castling_rights._as_string());
        println!("Board value: 0x{:X}", self.board(None).0);
        println!("Zobrist key: 0x{:X}", self.zobrist_key);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bitboard(u64);

impl Bitboard {
    pub fn new(bitboard: u64) -> Self {
        Bitboard(bitboard)
    }

    pub fn from_square(square: Square) -> Self {
        let mut bitboard = Self(0);
        bitboard.set_bit(square);

        bitboard
    }

    pub fn bit_occupied(self, square: Square) -> bool {
        self.0 & (1 << square as usize) != 0
    }

    pub fn set_bit(&mut self, square: Square) {
        self.0 |= 1 << square as usize;
    }

    pub fn pop_bit(&mut self, square: Square) {
        self.0 &= !(1 << square as usize);
    }

    pub fn value(self) -> u64 {
        self.0
    }

    // lsb = least significant bit
    pub fn get_lsb_square(self) -> Option<Square> {
        if self.0 == 0 {
            return None;
        }

        Square::from_u32(self.0.trailing_zeros())
    }

    pub fn _count_bits(self) -> u32 {
        self.0.count_ones()
    }

    fn _print(self) {
        for square in Square::iter() {
            if square.file() == 0 {
                print!("{:<4}", ((64 - square as usize) / 8));
            }

            print!("{:<2}", if self.bit_occupied(square) { 1 } else { 0 });

            if square.file() == 7 {
                println!();
            }
        }

        println!();
        println!("    a b c d e f g h");
        println!();
        println!("Bitboard decimal value: {}", self.0);
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl<T: Unsigned + AsPrimitive<u64>> BitAnd<T> for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: T) -> Self::Output {
        Self(self.0 & rhs.as_())
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = Self(self.0 & rhs.0)
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = Self(self.0 | rhs.0)
    }
}

impl<T: Unsigned + AsPrimitive<u64>> BitOrAssign<T> for Bitboard {
    fn bitor_assign(&mut self, rhs: T) {
        *self = Self(self.0 | rhs.as_())
    }
}

impl Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl<T: Unsigned + AsPrimitive<u64>> PartialEq<T> for Bitboard {
    fn eq(&self, other: &T) -> bool {
        self.0 == other.as_()
    }
}

impl<T: Unsigned + AsPrimitive<u64>> Shl<T> for Bitboard {
    type Output = Self;

    fn shl(self, rhs: T) -> Self::Output {
        Self(self.0 << rhs.as_())
    }
}

impl<T: Unsigned + AsPrimitive<u64>> Shr<T> for Bitboard {
    type Output = Self;

    fn shr(self, rhs: T) -> Self::Output {
        Self(self.0 >> rhs.as_())
    }
}

impl<T: Unsigned + AsPrimitive<u64>> ShrAssign<T> for Bitboard {
    fn shr_assign(&mut self, rhs: T) {
        *self = Self(self.0 >> rhs.as_())
    }
}

#[derive(Clone, Copy, Debug, EnumIter, FromPrimitive, PartialEq)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Piece {
    pub fn from_char(character: char) -> Result<Self, ParseError> {
        match character {
            'P' | 'p' => Ok(Self::Pawn),
            'N' | 'n' => Ok(Self::Knight),
            'B' | 'b' => Ok(Self::Bishop),
            'R' | 'r' => Ok(Self::Rook),
            'Q' | 'q' => Ok(Self::Queen),
            'K' | 'k' => Ok(Self::King),
            _ => Err(ParseError::VariantNotFound),
        }
    }

    pub fn _to_char(self, side: Option<Side>) -> char {
        match side {
            Some(side) => match side {
                Side::White => match self {
                    Self::Pawn => 'P',
                    Self::Knight => 'N',
                    Self::Bishop => 'B',
                    Self::Rook => 'R',
                    Self::Queen => 'Q',
                    Self::King => 'K',
                },
                Side::Black => match self {
                    Self::Pawn => 'p',
                    Self::Knight => 'n',
                    Self::Bishop => 'b',
                    Self::Rook => 'r',
                    Self::Queen => 'q',
                    Self::King => 'k',
                },
            },
            None => match self {
                Self::Pawn => 'p',
                Self::Knight => 'n',
                Self::Bishop => 'b',
                Self::Rook => 'r',
                Self::Queen => 'q',
                Self::King => 'k',
            },
        }
    }
}

#[derive(Clone, Copy, Debug, EnumIter, PartialEq)]
pub enum Side {
    White,
    Black,
}

impl Side {
    pub fn opponent_side(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }

    pub fn to_value(self) -> Value {
        match self {
            Self::White => 1,
            Self::Black => -1,
        }
    }
}

#[rustfmt::skip]
#[derive(Clone, Copy, Debug, Display, EnumIter, EnumString, FromPrimitive, PartialEq)]
pub enum Square {
    A8, B8, C8, D8, E8, F8, G8, H8,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A1, B1, C1, D1, E1, F1, G1, H1,
}

impl Square {
    pub fn from_rank_file(rank: usize, file: usize) -> Self {
        Self::from_usize(rank * 8 + file).unwrap()
    }

    pub fn rank(self) -> usize {
        self as usize / 8
    }

    pub fn file(self) -> usize {
        self as usize % 8
    }

    pub fn horizontal_mirror(self) -> Square {
        let mirror_rank = 7 - self.rank();
        let square_index = mirror_rank * 8 + self.file();

        Square::from_usize(square_index).unwrap()
    }

    pub fn _to_lowercase_string(self) -> String {
        self.to_string().to_lowercase()
    }
}

#[derive(Clone, Copy)]
pub enum CastlingType {
    WhiteShort = 0b0001,
    WhiteLong = 0b0010,
    BlackShort = 0b0100,
    BlackLong = 0b1000,
}

impl CastlingType {
    pub fn _move_string(&self) -> &str {
        match self {
            Self::WhiteShort => "e1g1",
            Self::WhiteLong => "e1c1",
            Self::BlackShort => "e8g8",
            Self::BlackLong => "e8c8",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct CastlingRights(u8);

impl CastlingRights {
    fn initialise(castling_rights_string: &str) -> Result<Self, InputError> {
        if castling_rights_string == "-" {
            return Ok(Self(0));
        };

        let mut castling_rights = 0;

        for character in castling_rights_string.chars() {
            match character {
                'K' => castling_rights |= CastlingType::WhiteShort as u8,
                'Q' => castling_rights |= CastlingType::WhiteLong as u8,
                'k' => castling_rights |= CastlingType::BlackShort as u8,
                'q' => castling_rights |= CastlingType::BlackLong as u8,
                _ => return Err(InputError::InvalidFen(FenError::CastlingRights)),
            }
        }

        Ok(Self(castling_rights))
    }

    fn remove_castling_type(&mut self, castling_type: CastlingType) {
        self.0 &= !(castling_type as u8);
    }

    fn _as_string(&self) -> String {
        let mut castling_rights_string = String::new();

        if self.0 & CastlingType::WhiteShort as u8 != 0 {
            castling_rights_string.push('K');
        }

        if self.0 & CastlingType::WhiteLong as u8 != 0 {
            castling_rights_string.push('Q');
        }

        if self.0 & CastlingType::BlackShort as u8 != 0 {
            castling_rights_string.push('k');
        }

        if self.0 & CastlingType::BlackLong as u8 != 0 {
            castling_rights_string.push('q');
        }

        castling_rights_string
    }
}

impl<T: Unsigned + AsPrimitive<u8>> PartialEq<T> for CastlingRights {
    fn eq(&self, other: &T) -> bool {
        self.0 == other.as_()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::moves::{MoveList, MoveSearch},
        *,
    };

    #[test]
    fn load_start_position() {
        let mut game = Game::initialise();
        let fen = vec!["startpos"];
        game.load_fen(&fen).unwrap();

        let mut desired_white_pawns_bitboard = Bitboard(0);
        desired_white_pawns_bitboard.set_bit(Square::A2);
        desired_white_pawns_bitboard.set_bit(Square::B2);
        desired_white_pawns_bitboard.set_bit(Square::C2);
        desired_white_pawns_bitboard.set_bit(Square::D2);
        desired_white_pawns_bitboard.set_bit(Square::E2);
        desired_white_pawns_bitboard.set_bit(Square::F2);
        desired_white_pawns_bitboard.set_bit(Square::G2);
        desired_white_pawns_bitboard.set_bit(Square::H2);

        let mut desired_white_knights_bitboard = Bitboard(0);
        desired_white_knights_bitboard.set_bit(Square::B1);
        desired_white_knights_bitboard.set_bit(Square::G1);

        let mut desired_white_bishops_bitboard = Bitboard(0);
        desired_white_bishops_bitboard.set_bit(Square::C1);
        desired_white_bishops_bitboard.set_bit(Square::F1);

        let mut desired_white_rooks_bitboard = Bitboard(0);
        desired_white_rooks_bitboard.set_bit(Square::A1);
        desired_white_rooks_bitboard.set_bit(Square::H1);

        let mut desired_white_queens_bitboard = Bitboard(0);
        desired_white_queens_bitboard.set_bit(Square::D1);

        let mut desired_white_king_bitboard = Bitboard(0);
        desired_white_king_bitboard.set_bit(Square::E1);

        let mut desired_black_pawns_bitboard = Bitboard(0);
        desired_black_pawns_bitboard.set_bit(Square::A7);
        desired_black_pawns_bitboard.set_bit(Square::B7);
        desired_black_pawns_bitboard.set_bit(Square::C7);
        desired_black_pawns_bitboard.set_bit(Square::D7);
        desired_black_pawns_bitboard.set_bit(Square::E7);
        desired_black_pawns_bitboard.set_bit(Square::F7);
        desired_black_pawns_bitboard.set_bit(Square::G7);
        desired_black_pawns_bitboard.set_bit(Square::H7);

        let mut desired_black_knights_bitboard = Bitboard(0);
        desired_black_knights_bitboard.set_bit(Square::B8);
        desired_black_knights_bitboard.set_bit(Square::G8);

        let mut desired_black_bishops_bitboard = Bitboard(0);
        desired_black_bishops_bitboard.set_bit(Square::C8);
        desired_black_bishops_bitboard.set_bit(Square::F8);

        let mut desired_black_rooks_bitboard = Bitboard(0);
        desired_black_rooks_bitboard.set_bit(Square::A8);
        desired_black_rooks_bitboard.set_bit(Square::H8);

        let mut desired_black_queens_bitboard = Bitboard(0);
        desired_black_queens_bitboard.set_bit(Square::D8);

        let mut desired_black_king_bitboard = Bitboard(0);
        desired_black_king_bitboard.set_bit(Square::E8);

        let desired_side_to_move = Side::White;
        let desired_castling_rights = CastlingRights(0b1111);
        let desired_en_passant_square = None;
        let desired_halfmove_clock = 0;

        assert_eq!(game.white_pawns, desired_white_pawns_bitboard);
        assert_eq!(game.white_knights, desired_white_knights_bitboard);
        assert_eq!(game.white_bishops, desired_white_bishops_bitboard);
        assert_eq!(game.white_rooks, desired_white_rooks_bitboard);
        assert_eq!(game.white_queens, desired_white_queens_bitboard);
        assert_eq!(game.white_king, desired_white_king_bitboard);

        assert_eq!(game.black_pawns, desired_black_pawns_bitboard);
        assert_eq!(game.black_knights, desired_black_knights_bitboard);
        assert_eq!(game.black_bishops, desired_black_bishops_bitboard);
        assert_eq!(game.black_rooks, desired_black_rooks_bitboard);
        assert_eq!(game.black_queens, desired_black_queens_bitboard);
        assert_eq!(game.black_king, desired_black_king_bitboard);

        assert_eq!(game.side_to_move, desired_side_to_move);
        assert_eq!(game.castling_rights, desired_castling_rights);
        assert_eq!(game.en_passant_square, desired_en_passant_square);
        assert_eq!(game.halfmove_clock, desired_halfmove_clock);
    }

    #[test]
    fn load_tricky_position() {
        let mut game = Game::initialise();
        let fen = vec![
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R",
            "w",
            "KQkq",
            "-",
            "0",
            "1",
        ];
        game.load_fen(&fen).unwrap();

        let mut desired_white_pawns_bitboard = Bitboard(0);
        desired_white_pawns_bitboard.set_bit(Square::A2);
        desired_white_pawns_bitboard.set_bit(Square::B2);
        desired_white_pawns_bitboard.set_bit(Square::C2);
        desired_white_pawns_bitboard.set_bit(Square::D5);
        desired_white_pawns_bitboard.set_bit(Square::E4);
        desired_white_pawns_bitboard.set_bit(Square::F2);
        desired_white_pawns_bitboard.set_bit(Square::G2);
        desired_white_pawns_bitboard.set_bit(Square::H2);

        let mut desired_white_knights_bitboard = Bitboard(0);
        desired_white_knights_bitboard.set_bit(Square::C3);
        desired_white_knights_bitboard.set_bit(Square::E5);

        let mut desired_white_bishops_bitboard = Bitboard(0);
        desired_white_bishops_bitboard.set_bit(Square::D2);
        desired_white_bishops_bitboard.set_bit(Square::E2);

        let mut desired_white_rooks_bitboard = Bitboard(0);
        desired_white_rooks_bitboard.set_bit(Square::A1);
        desired_white_rooks_bitboard.set_bit(Square::H1);

        let mut desired_white_queens_bitboard = Bitboard(0);
        desired_white_queens_bitboard.set_bit(Square::F3);

        let mut desired_white_king_bitboard = Bitboard(0);
        desired_white_king_bitboard.set_bit(Square::E1);

        let mut desired_black_pawns_bitboard = Bitboard(0);
        desired_black_pawns_bitboard.set_bit(Square::A7);
        desired_black_pawns_bitboard.set_bit(Square::B4);
        desired_black_pawns_bitboard.set_bit(Square::C7);
        desired_black_pawns_bitboard.set_bit(Square::D7);
        desired_black_pawns_bitboard.set_bit(Square::E6);
        desired_black_pawns_bitboard.set_bit(Square::F7);
        desired_black_pawns_bitboard.set_bit(Square::G6);
        desired_black_pawns_bitboard.set_bit(Square::H3);

        let mut desired_black_knights_bitboard = Bitboard(0);
        desired_black_knights_bitboard.set_bit(Square::B6);
        desired_black_knights_bitboard.set_bit(Square::F6);

        let mut desired_black_bishops_bitboard = Bitboard(0);
        desired_black_bishops_bitboard.set_bit(Square::A6);
        desired_black_bishops_bitboard.set_bit(Square::G7);

        let mut desired_black_rooks_bitboard = Bitboard(0);
        desired_black_rooks_bitboard.set_bit(Square::A8);
        desired_black_rooks_bitboard.set_bit(Square::H8);

        let mut desired_black_queens_bitboard = Bitboard(0);
        desired_black_queens_bitboard.set_bit(Square::E7);

        let mut desired_black_king_bitboard = Bitboard(0);
        desired_black_king_bitboard.set_bit(Square::E8);

        let desired_side_to_move = Side::White;
        let desired_castling_rights = CastlingRights(0b1111);
        let desired_en_passant_square = None;
        let desired_halfmove_clock = 0;

        assert_eq!(game.white_pawns, desired_white_pawns_bitboard);
        assert_eq!(game.white_knights, desired_white_knights_bitboard);
        assert_eq!(game.white_bishops, desired_white_bishops_bitboard);
        assert_eq!(game.white_rooks, desired_white_rooks_bitboard);
        assert_eq!(game.white_queens, desired_white_queens_bitboard);
        assert_eq!(game.white_king, desired_white_king_bitboard);

        assert_eq!(game.black_pawns, desired_black_pawns_bitboard);
        assert_eq!(game.black_knights, desired_black_knights_bitboard);
        assert_eq!(game.black_bishops, desired_black_bishops_bitboard);
        assert_eq!(game.black_rooks, desired_black_rooks_bitboard);
        assert_eq!(game.black_queens, desired_black_queens_bitboard);
        assert_eq!(game.black_king, desired_black_king_bitboard);

        assert_eq!(game.side_to_move, desired_side_to_move);
        assert_eq!(game.castling_rights, desired_castling_rights);
        assert_eq!(game.en_passant_square, desired_en_passant_square);
        assert_eq!(game.halfmove_clock, desired_halfmove_clock);
    }

    #[test]
    fn load_killer_position() {
        let mut game = Game::initialise();
        let fen = vec![
            "rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/3P3P/P1P1P3/RNBQKBNR",
            "w",
            "KQkq",
            "e6",
            "0",
            "1",
        ];
        game.load_fen(&fen).unwrap();

        let mut desired_white_pawns_bitboard = Bitboard(0);
        desired_white_pawns_bitboard.set_bit(Square::A2);
        desired_white_pawns_bitboard.set_bit(Square::B4);
        desired_white_pawns_bitboard.set_bit(Square::C2);
        desired_white_pawns_bitboard.set_bit(Square::D3);
        desired_white_pawns_bitboard.set_bit(Square::D4);
        desired_white_pawns_bitboard.set_bit(Square::E2);
        desired_white_pawns_bitboard.set_bit(Square::F5);
        desired_white_pawns_bitboard.set_bit(Square::G7);
        desired_white_pawns_bitboard.set_bit(Square::H3);

        let mut desired_white_knights_bitboard = Bitboard(0);
        desired_white_knights_bitboard.set_bit(Square::B1);
        desired_white_knights_bitboard.set_bit(Square::G1);

        let mut desired_white_bishops_bitboard = Bitboard(0);
        desired_white_bishops_bitboard.set_bit(Square::C1);
        desired_white_bishops_bitboard.set_bit(Square::F1);

        let mut desired_white_rooks_bitboard = Bitboard(0);
        desired_white_rooks_bitboard.set_bit(Square::A1);
        desired_white_rooks_bitboard.set_bit(Square::H1);

        let mut desired_white_queens_bitboard = Bitboard(0);
        desired_white_queens_bitboard.set_bit(Square::D1);

        let mut desired_white_king_bitboard = Bitboard(0);
        desired_white_king_bitboard.set_bit(Square::E1);

        let mut desired_black_pawns_bitboard = Bitboard(0);
        desired_black_pawns_bitboard.set_bit(Square::A7);
        desired_black_pawns_bitboard.set_bit(Square::B7);
        desired_black_pawns_bitboard.set_bit(Square::C5);
        desired_black_pawns_bitboard.set_bit(Square::D7);
        desired_black_pawns_bitboard.set_bit(Square::E5);
        desired_black_pawns_bitboard.set_bit(Square::F7);
        desired_black_pawns_bitboard.set_bit(Square::H7);

        let mut desired_black_knights_bitboard = Bitboard(0);
        desired_black_knights_bitboard.set_bit(Square::B8);

        let mut desired_black_bishops_bitboard = Bitboard(0);
        desired_black_bishops_bitboard.set_bit(Square::C8);
        desired_black_bishops_bitboard.set_bit(Square::F8);

        let mut desired_black_rooks_bitboard = Bitboard(0);
        desired_black_rooks_bitboard.set_bit(Square::A8);
        desired_black_rooks_bitboard.set_bit(Square::H8);

        let mut desired_black_queens_bitboard = Bitboard(0);
        desired_black_queens_bitboard.set_bit(Square::D8);

        let mut desired_black_king_bitboard = Bitboard(0);
        desired_black_king_bitboard.set_bit(Square::E8);

        let desired_side_to_move = Side::White;
        let desired_castling_rights = CastlingRights(0b1111);
        let desired_en_passant_square = Some(Square::E6);
        let desired_halfmove_clock = 0;

        assert_eq!(game.white_pawns, desired_white_pawns_bitboard);
        assert_eq!(game.white_knights, desired_white_knights_bitboard);
        assert_eq!(game.white_bishops, desired_white_bishops_bitboard);
        assert_eq!(game.white_rooks, desired_white_rooks_bitboard);
        assert_eq!(game.white_queens, desired_white_queens_bitboard);
        assert_eq!(game.white_king, desired_white_king_bitboard);

        assert_eq!(game.black_pawns, desired_black_pawns_bitboard);
        assert_eq!(game.black_knights, desired_black_knights_bitboard);
        assert_eq!(game.black_bishops, desired_black_bishops_bitboard);
        assert_eq!(game.black_rooks, desired_black_rooks_bitboard);
        assert_eq!(game.black_queens, desired_black_queens_bitboard);
        assert_eq!(game.black_king, desired_black_king_bitboard);

        assert_eq!(game.side_to_move, desired_side_to_move);
        assert_eq!(game.castling_rights, desired_castling_rights);
        assert_eq!(game.en_passant_square, desired_en_passant_square);
        assert_eq!(game.halfmove_clock, desired_halfmove_clock);
    }

    #[test]
    fn load_cmk_position() {
        let mut game = Game::initialise();
        let fen = vec![
            "r2q1rk1/ppp2ppp/2n1bn2/2b1p3/3pP3/3P1NPP/PPP1NPB1/R1BQ1RK1",
            "b",
            "-",
            "-",
            "0",
            "9",
        ];
        game.load_fen(&fen).unwrap();

        let mut desired_white_pawns_bitboard = Bitboard(0);
        desired_white_pawns_bitboard.set_bit(Square::A2);
        desired_white_pawns_bitboard.set_bit(Square::B2);
        desired_white_pawns_bitboard.set_bit(Square::C2);
        desired_white_pawns_bitboard.set_bit(Square::D3);
        desired_white_pawns_bitboard.set_bit(Square::E4);
        desired_white_pawns_bitboard.set_bit(Square::F2);
        desired_white_pawns_bitboard.set_bit(Square::G3);
        desired_white_pawns_bitboard.set_bit(Square::H3);

        let mut desired_white_knights_bitboard = Bitboard(0);
        desired_white_knights_bitboard.set_bit(Square::E2);
        desired_white_knights_bitboard.set_bit(Square::F3);

        let mut desired_white_bishops_bitboard = Bitboard(0);
        desired_white_bishops_bitboard.set_bit(Square::C1);
        desired_white_bishops_bitboard.set_bit(Square::G2);

        let mut desired_white_rooks_bitboard = Bitboard(0);
        desired_white_rooks_bitboard.set_bit(Square::A1);
        desired_white_rooks_bitboard.set_bit(Square::F1);

        let mut desired_white_queens_bitboard = Bitboard(0);
        desired_white_queens_bitboard.set_bit(Square::D1);

        let mut desired_white_king_bitboard = Bitboard(0);
        desired_white_king_bitboard.set_bit(Square::G1);

        let mut desired_black_pawns_bitboard = Bitboard(0);
        desired_black_pawns_bitboard.set_bit(Square::A7);
        desired_black_pawns_bitboard.set_bit(Square::B7);
        desired_black_pawns_bitboard.set_bit(Square::C7);
        desired_black_pawns_bitboard.set_bit(Square::D4);
        desired_black_pawns_bitboard.set_bit(Square::E5);
        desired_black_pawns_bitboard.set_bit(Square::F7);
        desired_black_pawns_bitboard.set_bit(Square::G7);
        desired_black_pawns_bitboard.set_bit(Square::H7);

        let mut desired_black_knights_bitboard = Bitboard(0);
        desired_black_knights_bitboard.set_bit(Square::C6);
        desired_black_knights_bitboard.set_bit(Square::F6);

        let mut desired_black_bishops_bitboard = Bitboard(0);
        desired_black_bishops_bitboard.set_bit(Square::C5);
        desired_black_bishops_bitboard.set_bit(Square::E6);

        let mut desired_black_rooks_bitboard = Bitboard(0);
        desired_black_rooks_bitboard.set_bit(Square::A8);
        desired_black_rooks_bitboard.set_bit(Square::F8);

        let mut desired_black_queens_bitboard = Bitboard(0);
        desired_black_queens_bitboard.set_bit(Square::D8);

        let mut desired_black_king_bitboard = Bitboard(0);
        desired_black_king_bitboard.set_bit(Square::G8);

        let desired_side_to_move = Side::Black;
        let desired_castling_rights = CastlingRights(0b0000);
        let desired_en_passant_square = None;
        let desired_halfmove_clock = 0;

        assert_eq!(game.white_pawns, desired_white_pawns_bitboard);
        assert_eq!(game.white_knights, desired_white_knights_bitboard);
        assert_eq!(game.white_bishops, desired_white_bishops_bitboard);
        assert_eq!(game.white_rooks, desired_white_rooks_bitboard);
        assert_eq!(game.white_queens, desired_white_queens_bitboard);
        assert_eq!(game.white_king, desired_white_king_bitboard);

        assert_eq!(game.black_pawns, desired_black_pawns_bitboard);
        assert_eq!(game.black_knights, desired_black_knights_bitboard);
        assert_eq!(game.black_bishops, desired_black_bishops_bitboard);
        assert_eq!(game.black_rooks, desired_black_rooks_bitboard);
        assert_eq!(game.black_queens, desired_black_queens_bitboard);
        assert_eq!(game.black_king, desired_black_king_bitboard);

        assert_eq!(game.side_to_move, desired_side_to_move);
        assert_eq!(game.castling_rights, desired_castling_rights);
        assert_eq!(game.en_passant_square, desired_en_passant_square);
        assert_eq!(game.halfmove_clock, desired_halfmove_clock);
    }

    #[test]
    fn start_position_moves() {
        let mut game = Game::initialise();
        let fen = vec!["startpos"];
        game.load_fen(&fen).unwrap();

        let mut desired_white_pawns_bitboard = Bitboard(0);
        desired_white_pawns_bitboard.set_bit(Square::A2);
        desired_white_pawns_bitboard.set_bit(Square::B2);
        desired_white_pawns_bitboard.set_bit(Square::C2);
        desired_white_pawns_bitboard.set_bit(Square::D2);
        desired_white_pawns_bitboard.set_bit(Square::E2);
        desired_white_pawns_bitboard.set_bit(Square::F2);
        desired_white_pawns_bitboard.set_bit(Square::G2);
        desired_white_pawns_bitboard.set_bit(Square::H2);

        let mut desired_white_knights_bitboard = Bitboard(0);
        desired_white_knights_bitboard.set_bit(Square::B1);
        desired_white_knights_bitboard.set_bit(Square::G1);

        let mut desired_white_bishops_bitboard = Bitboard(0);
        desired_white_bishops_bitboard.set_bit(Square::C1);
        desired_white_bishops_bitboard.set_bit(Square::F1);

        let mut desired_white_rooks_bitboard = Bitboard(0);
        desired_white_rooks_bitboard.set_bit(Square::A1);
        desired_white_rooks_bitboard.set_bit(Square::H1);

        let mut desired_white_queens_bitboard = Bitboard(0);
        desired_white_queens_bitboard.set_bit(Square::D1);

        let mut desired_white_king_bitboard = Bitboard(0);
        desired_white_king_bitboard.set_bit(Square::E1);

        let mut desired_black_pawns_bitboard = Bitboard(0);
        desired_black_pawns_bitboard.set_bit(Square::A7);
        desired_black_pawns_bitboard.set_bit(Square::B7);
        desired_black_pawns_bitboard.set_bit(Square::C7);
        desired_black_pawns_bitboard.set_bit(Square::D7);
        desired_black_pawns_bitboard.set_bit(Square::E7);
        desired_black_pawns_bitboard.set_bit(Square::F7);
        desired_black_pawns_bitboard.set_bit(Square::G7);
        desired_black_pawns_bitboard.set_bit(Square::H7);

        let mut desired_black_knights_bitboard = Bitboard(0);
        desired_black_knights_bitboard.set_bit(Square::B8);
        desired_black_knights_bitboard.set_bit(Square::G8);

        let mut desired_black_bishops_bitboard = Bitboard(0);
        desired_black_bishops_bitboard.set_bit(Square::C8);
        desired_black_bishops_bitboard.set_bit(Square::F8);

        let mut desired_black_rooks_bitboard = Bitboard(0);
        desired_black_rooks_bitboard.set_bit(Square::A8);
        desired_black_rooks_bitboard.set_bit(Square::H8);

        let mut desired_black_queens_bitboard = Bitboard(0);
        desired_black_queens_bitboard.set_bit(Square::D8);

        let mut desired_black_king_bitboard = Bitboard(0);
        desired_black_king_bitboard.set_bit(Square::E8);

        let desired_side_to_move = Side::White;
        let desired_castling_rights = CastlingRights(0b1111);
        let desired_en_passant_square = None;
        let desired_halfmove_clock = 0;

        assert_eq!(game.white_pawns, desired_white_pawns_bitboard);
        assert_eq!(game.white_knights, desired_white_knights_bitboard);
        assert_eq!(game.white_bishops, desired_white_bishops_bitboard);
        assert_eq!(game.white_rooks, desired_white_rooks_bitboard);
        assert_eq!(game.white_queens, desired_white_queens_bitboard);
        assert_eq!(game.white_king, desired_white_king_bitboard);

        assert_eq!(game.black_pawns, desired_black_pawns_bitboard);
        assert_eq!(game.black_knights, desired_black_knights_bitboard);
        assert_eq!(game.black_bishops, desired_black_bishops_bitboard);
        assert_eq!(game.black_rooks, desired_black_rooks_bitboard);
        assert_eq!(game.black_queens, desired_black_queens_bitboard);
        assert_eq!(game.black_king, desired_black_king_bitboard);

        assert_eq!(game.side_to_move, desired_side_to_move);
        assert_eq!(game.castling_rights, desired_castling_rights);
        assert_eq!(game.en_passant_square, desired_en_passant_square);
        assert_eq!(game.halfmove_clock, desired_halfmove_clock);

        let attack_tables = AttackTables::initialise();
        let move_list = MoveList::generate_moves(&game, &attack_tables);
        let move_search = MoveSearch::new(Square::E2, Square::E4, None);
        let mv = move_list.find_move(move_search).unwrap();
        game.make_move(&mv, &attack_tables).unwrap();

        desired_white_pawns_bitboard.pop_bit(Square::E2);
        desired_white_pawns_bitboard.set_bit(Square::E4);

        let desired_side_to_move = Side::Black;
        let desired_castling_rights = CastlingRights(0b1111);
        let desired_en_passant_square = Some(Square::E3);
        let desired_halfmove_clock = 0;

        assert_eq!(game.white_pawns, desired_white_pawns_bitboard);
        assert_eq!(game.white_knights, desired_white_knights_bitboard);
        assert_eq!(game.white_bishops, desired_white_bishops_bitboard);
        assert_eq!(game.white_rooks, desired_white_rooks_bitboard);
        assert_eq!(game.white_queens, desired_white_queens_bitboard);
        assert_eq!(game.white_king, desired_white_king_bitboard);

        assert_eq!(game.black_pawns, desired_black_pawns_bitboard);
        assert_eq!(game.black_knights, desired_black_knights_bitboard);
        assert_eq!(game.black_bishops, desired_black_bishops_bitboard);
        assert_eq!(game.black_rooks, desired_black_rooks_bitboard);
        assert_eq!(game.black_queens, desired_black_queens_bitboard);
        assert_eq!(game.black_king, desired_black_king_bitboard);

        assert_eq!(game.side_to_move, desired_side_to_move);
        assert_eq!(game.castling_rights, desired_castling_rights);
        assert_eq!(game.en_passant_square, desired_en_passant_square);
        assert_eq!(game.halfmove_clock, desired_halfmove_clock);

        let move_list = MoveList::generate_moves(&game, &attack_tables);
        let move_search = MoveSearch::new(Square::E7, Square::E5, None);
        let mv = move_list.find_move(move_search).unwrap();
        game.make_move(&mv, &attack_tables).unwrap();

        desired_black_pawns_bitboard.pop_bit(Square::E7);
        desired_black_pawns_bitboard.set_bit(Square::E5);

        let desired_side_to_move = Side::White;
        let desired_castling_rights = CastlingRights(0b1111);
        let desired_en_passant_square = Some(Square::E6);
        let desired_halfmove_clock = 0;

        assert_eq!(game.white_pawns, desired_white_pawns_bitboard);
        assert_eq!(game.white_knights, desired_white_knights_bitboard);
        assert_eq!(game.white_bishops, desired_white_bishops_bitboard);
        assert_eq!(game.white_rooks, desired_white_rooks_bitboard);
        assert_eq!(game.white_queens, desired_white_queens_bitboard);
        assert_eq!(game.white_king, desired_white_king_bitboard);

        assert_eq!(game.black_pawns, desired_black_pawns_bitboard);
        assert_eq!(game.black_knights, desired_black_knights_bitboard);
        assert_eq!(game.black_bishops, desired_black_bishops_bitboard);
        assert_eq!(game.black_rooks, desired_black_rooks_bitboard);
        assert_eq!(game.black_queens, desired_black_queens_bitboard);
        assert_eq!(game.black_king, desired_black_king_bitboard);

        assert_eq!(game.side_to_move, desired_side_to_move);
        assert_eq!(game.castling_rights, desired_castling_rights);
        assert_eq!(game.en_passant_square, desired_en_passant_square);
        assert_eq!(game.halfmove_clock, desired_halfmove_clock);

        let move_list = MoveList::generate_moves(&game, &attack_tables);
        let move_search = MoveSearch::new(Square::G1, Square::F3, None);
        let mv = move_list.find_move(move_search).unwrap();
        game.make_move(&mv, &attack_tables).unwrap();

        desired_white_knights_bitboard.pop_bit(Square::G1);
        desired_white_knights_bitboard.set_bit(Square::F3);

        let desired_side_to_move = Side::Black;
        let desired_castling_rights = CastlingRights(0b1111);
        let desired_en_passant_square = None;
        let desired_halfmove_clock = 0;

        assert_eq!(game.white_pawns, desired_white_pawns_bitboard);
        assert_eq!(game.white_knights, desired_white_knights_bitboard);
        assert_eq!(game.white_bishops, desired_white_bishops_bitboard);
        assert_eq!(game.white_rooks, desired_white_rooks_bitboard);
        assert_eq!(game.white_queens, desired_white_queens_bitboard);
        assert_eq!(game.white_king, desired_white_king_bitboard);

        assert_eq!(game.black_pawns, desired_black_pawns_bitboard);
        assert_eq!(game.black_knights, desired_black_knights_bitboard);
        assert_eq!(game.black_bishops, desired_black_bishops_bitboard);
        assert_eq!(game.black_rooks, desired_black_rooks_bitboard);
        assert_eq!(game.black_queens, desired_black_queens_bitboard);
        assert_eq!(game.black_king, desired_black_king_bitboard);

        assert_eq!(game.side_to_move, desired_side_to_move);
        assert_eq!(game.castling_rights, desired_castling_rights);
        assert_eq!(game.en_passant_square, desired_en_passant_square);
        assert_eq!(game.halfmove_clock, desired_halfmove_clock);
    }

    #[test]
    fn tricky_position_moves() {
        let mut game = Game::initialise();
        let fen = vec![
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R",
            "w",
            "KQkq",
            "-",
            "0",
            "1",
        ];
        game.load_fen(&fen).unwrap();

        let mut desired_white_pawns_bitboard = Bitboard(0);
        desired_white_pawns_bitboard.set_bit(Square::A2);
        desired_white_pawns_bitboard.set_bit(Square::B2);
        desired_white_pawns_bitboard.set_bit(Square::C2);
        desired_white_pawns_bitboard.set_bit(Square::D5);
        desired_white_pawns_bitboard.set_bit(Square::E4);
        desired_white_pawns_bitboard.set_bit(Square::F2);
        desired_white_pawns_bitboard.set_bit(Square::G2);
        desired_white_pawns_bitboard.set_bit(Square::H2);

        let mut desired_white_knights_bitboard = Bitboard(0);
        desired_white_knights_bitboard.set_bit(Square::C3);
        desired_white_knights_bitboard.set_bit(Square::E5);

        let mut desired_white_bishops_bitboard = Bitboard(0);
        desired_white_bishops_bitboard.set_bit(Square::D2);
        desired_white_bishops_bitboard.set_bit(Square::E2);

        let mut desired_white_rooks_bitboard = Bitboard(0);
        desired_white_rooks_bitboard.set_bit(Square::A1);
        desired_white_rooks_bitboard.set_bit(Square::H1);

        let mut desired_white_queens_bitboard = Bitboard(0);
        desired_white_queens_bitboard.set_bit(Square::F3);

        let mut desired_white_king_bitboard = Bitboard(0);
        desired_white_king_bitboard.set_bit(Square::E1);

        let mut desired_black_pawns_bitboard = Bitboard(0);
        desired_black_pawns_bitboard.set_bit(Square::A7);
        desired_black_pawns_bitboard.set_bit(Square::B4);
        desired_black_pawns_bitboard.set_bit(Square::C7);
        desired_black_pawns_bitboard.set_bit(Square::D7);
        desired_black_pawns_bitboard.set_bit(Square::E6);
        desired_black_pawns_bitboard.set_bit(Square::F7);
        desired_black_pawns_bitboard.set_bit(Square::G6);
        desired_black_pawns_bitboard.set_bit(Square::H3);

        let mut desired_black_knights_bitboard = Bitboard(0);
        desired_black_knights_bitboard.set_bit(Square::B6);
        desired_black_knights_bitboard.set_bit(Square::F6);

        let mut desired_black_bishops_bitboard = Bitboard(0);
        desired_black_bishops_bitboard.set_bit(Square::A6);
        desired_black_bishops_bitboard.set_bit(Square::G7);

        let mut desired_black_rooks_bitboard = Bitboard(0);
        desired_black_rooks_bitboard.set_bit(Square::A8);
        desired_black_rooks_bitboard.set_bit(Square::H8);

        let mut desired_black_queens_bitboard = Bitboard(0);
        desired_black_queens_bitboard.set_bit(Square::E7);

        let mut desired_black_king_bitboard = Bitboard(0);
        desired_black_king_bitboard.set_bit(Square::E8);

        let desired_side_to_move = Side::White;
        let desired_castling_rights = CastlingRights(0b1111);
        let desired_en_passant_square = None;
        let desired_halfmove_clock = 0;

        assert_eq!(game.white_pawns, desired_white_pawns_bitboard);
        assert_eq!(game.white_knights, desired_white_knights_bitboard);
        assert_eq!(game.white_bishops, desired_white_bishops_bitboard);
        assert_eq!(game.white_rooks, desired_white_rooks_bitboard);
        assert_eq!(game.white_queens, desired_white_queens_bitboard);
        assert_eq!(game.white_king, desired_white_king_bitboard);

        assert_eq!(game.black_pawns, desired_black_pawns_bitboard);
        assert_eq!(game.black_knights, desired_black_knights_bitboard);
        assert_eq!(game.black_bishops, desired_black_bishops_bitboard);
        assert_eq!(game.black_rooks, desired_black_rooks_bitboard);
        assert_eq!(game.black_queens, desired_black_queens_bitboard);
        assert_eq!(game.black_king, desired_black_king_bitboard);

        assert_eq!(game.side_to_move, desired_side_to_move);
        assert_eq!(game.castling_rights, desired_castling_rights);
        assert_eq!(game.en_passant_square, desired_en_passant_square);
        assert_eq!(game.halfmove_clock, desired_halfmove_clock);

        let attack_tables = AttackTables::initialise();
        let move_list = MoveList::generate_moves(&game, &attack_tables);
        let move_search = MoveSearch::new(Square::D5, Square::E6, None);
        let mv = move_list.find_move(move_search).unwrap();
        game.make_move(&mv, &attack_tables).unwrap();

        desired_white_pawns_bitboard.pop_bit(Square::D5);
        desired_white_pawns_bitboard.set_bit(Square::E6);
        desired_black_pawns_bitboard.pop_bit(Square::E6);

        let desired_side_to_move = Side::Black;
        let desired_castling_rights = CastlingRights(0b1111);
        let desired_en_passant_square = None;
        let desired_halfmove_clock = 0;

        assert_eq!(game.white_pawns, desired_white_pawns_bitboard);
        assert_eq!(game.white_knights, desired_white_knights_bitboard);
        assert_eq!(game.white_bishops, desired_white_bishops_bitboard);
        assert_eq!(game.white_rooks, desired_white_rooks_bitboard);
        assert_eq!(game.white_queens, desired_white_queens_bitboard);
        assert_eq!(game.white_king, desired_white_king_bitboard);

        assert_eq!(game.black_pawns, desired_black_pawns_bitboard);
        assert_eq!(game.black_knights, desired_black_knights_bitboard);
        assert_eq!(game.black_bishops, desired_black_bishops_bitboard);
        assert_eq!(game.black_rooks, desired_black_rooks_bitboard);
        assert_eq!(game.black_queens, desired_black_queens_bitboard);
        assert_eq!(game.black_king, desired_black_king_bitboard);

        assert_eq!(game.side_to_move, desired_side_to_move);
        assert_eq!(game.castling_rights, desired_castling_rights);
        assert_eq!(game.en_passant_square, desired_en_passant_square);
        assert_eq!(game.halfmove_clock, desired_halfmove_clock);

        let move_list = MoveList::generate_moves(&game, &attack_tables);
        let move_search = MoveSearch::new(Square::A6, Square::E2, None);
        let mv = move_list.find_move(move_search).unwrap();
        game.make_move(&mv, &attack_tables).unwrap();

        desired_black_bishops_bitboard.pop_bit(Square::A6);
        desired_black_bishops_bitboard.set_bit(Square::E2);
        desired_white_bishops_bitboard.pop_bit(Square::E2);

        let desired_side_to_move = Side::White;
        let desired_castling_rights = CastlingRights(0b1111);
        let desired_en_passant_square = None;
        let desired_halfmove_clock = 0;

        assert_eq!(game.white_pawns, desired_white_pawns_bitboard);
        assert_eq!(game.white_knights, desired_white_knights_bitboard);
        assert_eq!(game.white_bishops, desired_white_bishops_bitboard);
        assert_eq!(game.white_rooks, desired_white_rooks_bitboard);
        assert_eq!(game.white_queens, desired_white_queens_bitboard);
        assert_eq!(game.white_king, desired_white_king_bitboard);

        assert_eq!(game.black_pawns, desired_black_pawns_bitboard);
        assert_eq!(game.black_knights, desired_black_knights_bitboard);
        assert_eq!(game.black_bishops, desired_black_bishops_bitboard);
        assert_eq!(game.black_rooks, desired_black_rooks_bitboard);
        assert_eq!(game.black_queens, desired_black_queens_bitboard);
        assert_eq!(game.black_king, desired_black_king_bitboard);

        assert_eq!(game.side_to_move, desired_side_to_move);
        assert_eq!(game.castling_rights, desired_castling_rights);
        assert_eq!(game.en_passant_square, desired_en_passant_square);
        assert_eq!(game.halfmove_clock, desired_halfmove_clock);

        let move_list = MoveList::generate_moves(&game, &attack_tables);
        let move_search = MoveSearch::new(Square::C3, Square::E2, None);
        let mv = move_list.find_move(move_search).unwrap();
        game.make_move(&mv, &attack_tables).unwrap();

        desired_white_knights_bitboard.pop_bit(Square::C3);
        desired_white_knights_bitboard.set_bit(Square::E2);
        desired_black_bishops_bitboard.pop_bit(Square::E2);

        let desired_side_to_move = Side::Black;
        let desired_castling_rights = CastlingRights(0b1111);
        let desired_en_passant_square = None;
        let desired_halfmove_clock = 0;

        assert_eq!(game.white_pawns, desired_white_pawns_bitboard);
        assert_eq!(game.white_knights, desired_white_knights_bitboard);
        assert_eq!(game.white_bishops, desired_white_bishops_bitboard);
        assert_eq!(game.white_rooks, desired_white_rooks_bitboard);
        assert_eq!(game.white_queens, desired_white_queens_bitboard);
        assert_eq!(game.white_king, desired_white_king_bitboard);

        assert_eq!(game.black_pawns, desired_black_pawns_bitboard);
        assert_eq!(game.black_knights, desired_black_knights_bitboard);
        assert_eq!(game.black_bishops, desired_black_bishops_bitboard);
        assert_eq!(game.black_rooks, desired_black_rooks_bitboard);
        assert_eq!(game.black_queens, desired_black_queens_bitboard);
        assert_eq!(game.black_king, desired_black_king_bitboard);

        assert_eq!(game.side_to_move, desired_side_to_move);
        assert_eq!(game.castling_rights, desired_castling_rights);
        assert_eq!(game.en_passant_square, desired_en_passant_square);
        assert_eq!(game.halfmove_clock, desired_halfmove_clock);
    }

    #[test]
    fn zobrist_key_start_position() {
        let mut game = Game::initialise();
        let fen = vec!["startpos"];
        game.load_fen(&fen).unwrap();

        assert_eq!(game.zobrist_key, 0x6ED5_7B11_8AE9_9580);
    }

    #[test]
    fn zobrist_key_tricky_position() {
        let mut game = Game::initialise();
        let fen = vec![
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R",
            "w",
            "KQkq",
            "-",
            "0",
            "1",
        ];
        game.load_fen(&fen).unwrap();

        assert_eq!(game.zobrist_key, 0xB6A3_E0EE_0FF9_BED8);
    }

    #[test]
    fn zobrist_key_killer_position() {
        let mut game = Game::initialise();
        let fen = vec![
            "rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/3P3P/P1P1P3/RNBQKBNR",
            "w",
            "KQkq",
            "e6",
            "0",
            "1",
        ];
        game.load_fen(&fen).unwrap();

        assert_eq!(game.zobrist_key, 0xD153_C557_50DD_8681);
    }

    #[test]
    fn zobrist_key_cmk_position() {
        let mut game = Game::initialise();
        let fen = vec![
            "r2q1rk1/ppp2ppp/2n1bn2/2b1p3/3pP3/3P1NPP/PPP1NPB1/R1BQ1RK1",
            "b",
            "-",
            "-",
            "0",
            "9",
        ];
        game.load_fen(&fen).unwrap();

        assert_eq!(game.zobrist_key, 0x4F75_A469_51F3_92D4);
    }

    #[test]
    fn update_zobrist_key_quiet_move() {
        let mut game = Game::initialise();
        let fen = vec![
            "r2q1rk1/ppp2ppp/2n1bn2/2b1p3/3pP3/3P1NPP/PPP1NPB1/R1BQ1RK1",
            "b",
            "-",
            "-",
            "0",
            "9",
        ];
        game.load_fen(&fen).unwrap();

        let attack_tables = AttackTables::initialise();
        let move_list = MoveList::generate_moves(&game, &attack_tables);
        let move_search = MoveSearch::new(Square::F8, Square::E8, None);
        let mv = move_list.find_move(move_search).unwrap();
        game.make_move(&mv, &attack_tables).unwrap();

        let generated_key = zobrist_hashes::ZOBRIST_HASHES.generate_key(&game);

        assert_eq!(game.zobrist_key, generated_key);
    }

    #[test]
    fn update_zobrist_key_capture() {
        let mut game = Game::initialise();
        let fen = vec![
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R",
            "w",
            "KQkq",
            "-",
            "0",
            "1",
        ];
        game.load_fen(&fen).unwrap();

        let attack_tables = AttackTables::initialise();
        let move_list = MoveList::generate_moves(&game, &attack_tables);
        let move_search = MoveSearch::new(Square::E2, Square::A6, None);
        let mv = move_list.find_move(move_search).unwrap();
        game.make_move(&mv, &attack_tables).unwrap();

        let generated_key = zobrist_hashes::ZOBRIST_HASHES.generate_key(&game);

        assert_eq!(game.zobrist_key, generated_key);
    }

    #[test]
    fn update_zobrist_key_double_pawn_push() {
        let mut game = Game::initialise();
        let fen = vec!["startpos"];
        game.load_fen(&fen).unwrap();

        let attack_tables = AttackTables::initialise();
        let move_list = MoveList::generate_moves(&game, &attack_tables);
        let move_search = MoveSearch::new(Square::E2, Square::E4, None);
        let mv = move_list.find_move(move_search).unwrap();
        game.make_move(&mv, &attack_tables).unwrap();

        let generated_key = zobrist_hashes::ZOBRIST_HASHES.generate_key(&game);

        assert_eq!(game.zobrist_key, generated_key);

        let move_list = MoveList::generate_moves(&game, &attack_tables);
        let move_search = MoveSearch::new(Square::E7, Square::E5, None);
        let mv = move_list.find_move(move_search).unwrap();
        game.make_move(&mv, &attack_tables).unwrap();

        let generated_key = zobrist_hashes::ZOBRIST_HASHES.generate_key(&game);

        assert_eq!(game.zobrist_key, generated_key);
    }

    #[test]
    fn update_zobrist_key_en_passant() {
        let mut game = Game::initialise();
        let fen = vec![
            "rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/3P3P/P1P1P3/RNBQKBNR",
            "w",
            "KQkq",
            "e6",
            "0",
            "1",
        ];
        game.load_fen(&fen).unwrap();

        let attack_tables = AttackTables::initialise();
        let move_list = MoveList::generate_moves(&game, &attack_tables);
        let move_search = MoveSearch::new(Square::F5, Square::E6, None);
        let mv = move_list.find_move(move_search).unwrap();
        game.make_move(&mv, &attack_tables).unwrap();

        let generated_key = zobrist_hashes::ZOBRIST_HASHES.generate_key(&game);

        assert_eq!(game.zobrist_key, generated_key);
    }

    #[test]
    fn update_zobrist_key_castling_rights() {
        let mut game = Game::initialise();
        let fen = vec![
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R",
            "w",
            "KQkq",
            "-",
            "0",
            "1",
        ];
        game.load_fen(&fen).unwrap();

        // short castle
        let attack_tables = AttackTables::initialise();
        let move_list = MoveList::generate_moves(&game, &attack_tables);
        let move_search = MoveSearch::new(Square::E1, Square::G1, None);
        let mv = move_list.find_move(move_search).unwrap();
        game.make_move(&mv, &attack_tables).unwrap();

        let generated_key = zobrist_hashes::ZOBRIST_HASHES.generate_key(&game);

        assert_eq!(game.zobrist_key, generated_key);

        // long castle
        game.load_fen(&fen).unwrap();

        let move_search = MoveSearch::new(Square::E1, Square::C1, None);
        let mv = move_list.find_move(move_search).unwrap();
        game.make_move(&mv, &attack_tables).unwrap();

        let generated_key = zobrist_hashes::ZOBRIST_HASHES.generate_key(&game);

        assert_eq!(game.zobrist_key, generated_key);

        // king move
        game.load_fen(&fen).unwrap();

        let move_search = MoveSearch::new(Square::E1, Square::D1, None);
        let mv = move_list.find_move(move_search).unwrap();
        game.make_move(&mv, &attack_tables).unwrap();

        let generated_key = zobrist_hashes::ZOBRIST_HASHES.generate_key(&game);

        assert_eq!(game.zobrist_key, generated_key);

        // queenside rook move
        game.load_fen(&fen).unwrap();

        let move_search = MoveSearch::new(Square::A1, Square::B1, None);
        let mv = move_list.find_move(move_search).unwrap();
        game.make_move(&mv, &attack_tables).unwrap();

        let generated_key = zobrist_hashes::ZOBRIST_HASHES.generate_key(&game);

        assert_eq!(game.zobrist_key, generated_key);

        // kingside rook move
        game.load_fen(&fen).unwrap();

        let move_search = MoveSearch::new(Square::H1, Square::G1, None);
        let mv = move_list.find_move(move_search).unwrap();
        game.make_move(&mv, &attack_tables).unwrap();

        let generated_key = zobrist_hashes::ZOBRIST_HASHES.generate_key(&game);

        assert_eq!(game.zobrist_key, generated_key);

        // capture queenside rook
        let fen = vec![
            "rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/3P3P/P1P1P3/RNBQKBNR",
            "w",
            "KQkq",
            "e6",
            "0",
            "1",
        ];
        game.load_fen(&fen).unwrap();

        let move_list = MoveList::generate_moves(&game, &attack_tables);
        let move_search = MoveSearch::new(Square::G7, Square::H8, Some(Piece::Queen));
        let mv = move_list.find_move(move_search).unwrap();
        game.make_move(&mv, &attack_tables).unwrap();

        let generated_key = zobrist_hashes::ZOBRIST_HASHES.generate_key(&game);

        assert_eq!(game.zobrist_key, generated_key);

        // capture kingside rook
        let fen = vec![
            "rnbqkb1r/pP1p1p1p/8/2p1pP2/1P1P4/3P3P/P1P1P3/RNBQKBNR",
            "w",
            "KQkq",
            "e6",
            "0",
            "1",
        ];
        game.load_fen(&fen).unwrap();

        let move_list = MoveList::generate_moves(&game, &attack_tables);
        let move_search = MoveSearch::new(Square::B7, Square::A8, Some(Piece::Queen));
        let mv = move_list.find_move(move_search).unwrap();
        game.make_move(&mv, &attack_tables).unwrap();

        let generated_key = zobrist_hashes::ZOBRIST_HASHES.generate_key(&game);

        assert_eq!(game.zobrist_key, generated_key);
    }

    #[test]
    fn update_zobrist_key_promotion() {
        let mut game = Game::initialise();
        let fen = vec![
            "rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/3P3P/P1P1P3/RNBQKBNR",
            "w",
            "KQkq",
            "e6",
            "0",
            "1",
        ];
        game.load_fen(&fen).unwrap();

        let attack_tables = AttackTables::initialise();
        let move_list = MoveList::generate_moves(&game, &attack_tables);
        let move_search = MoveSearch::new(Square::G7, Square::H8, Some(Piece::Queen));
        let mv = move_list.find_move(move_search).unwrap();
        game.make_move(&mv, &attack_tables).unwrap();

        let generated_key = zobrist_hashes::ZOBRIST_HASHES.generate_key(&game);

        assert_eq!(game.zobrist_key, generated_key);

        game.load_fen(&fen).unwrap();

        let move_search = MoveSearch::new(Square::G7, Square::H8, Some(Piece::Rook));
        let mv = move_list.find_move(move_search).unwrap();
        game.make_move(&mv, &attack_tables).unwrap();

        let generated_key = zobrist_hashes::ZOBRIST_HASHES.generate_key(&game);

        assert_eq!(game.zobrist_key, generated_key);

        game.load_fen(&fen).unwrap();

        let move_search = MoveSearch::new(Square::G7, Square::H8, Some(Piece::Bishop));
        let mv = move_list.find_move(move_search).unwrap();
        game.make_move(&mv, &attack_tables).unwrap();

        let generated_key = zobrist_hashes::ZOBRIST_HASHES.generate_key(&game);

        assert_eq!(game.zobrist_key, generated_key);

        game.load_fen(&fen).unwrap();

        let move_search = MoveSearch::new(Square::G7, Square::H8, Some(Piece::Knight));
        let mv = move_list.find_move(move_search).unwrap();
        game.make_move(&mv, &attack_tables).unwrap();

        let generated_key = zobrist_hashes::ZOBRIST_HASHES.generate_key(&game);

        assert_eq!(game.zobrist_key, generated_key);
    }

    #[test]
    fn update_zobrist_key_null_move() {
        let mut game = Game::initialise();
        let fen = vec![
            "rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/3P3P/P1P1P3/RNBQKBNR",
            "w",
            "KQkq",
            "e6",
            "0",
            "1",
        ];
        game.load_fen(&fen).unwrap();
        game.make_null_move();

        let generated_key = zobrist_hashes::ZOBRIST_HASHES.generate_key(&game);

        assert_eq!(game.zobrist_key, generated_key);
    }
    #[test]
    fn set_bit() {
        let mut bitboard1 = Bitboard(0);
        bitboard1.set_bit(Square::H2);
        let mut bitboard2 = Bitboard(0);
        bitboard2.set_bit(Square::G6);
        let mut bitboard3 = Bitboard(0);
        bitboard3.set_bit(Square::B4);

        assert_eq!(bitboard1.0, u64::pow(2, Square::H2 as u32));
        assert_eq!(bitboard2.0, u64::pow(2, Square::G6 as u32));
        assert_eq!(bitboard3.0, u64::pow(2, Square::B4 as u32));
    }

    #[test]
    fn pop_bit() {
        let mut bitboard1 = Bitboard(0);
        bitboard1.set_bit(Square::G5);
        bitboard1.set_bit(Square::A8);
        bitboard1.pop_bit(Square::G5);
        let mut bitboard2 = Bitboard(0);
        bitboard2.set_bit(Square::C1);
        bitboard2.set_bit(Square::A7);
        bitboard2.pop_bit(Square::C1);
        let mut bitboard3 = Bitboard(0);
        bitboard3.set_bit(Square::C4);
        bitboard3.set_bit(Square::B8);
        bitboard3.pop_bit(Square::C4);

        assert_eq!(bitboard1.0, u64::pow(2, Square::A8 as u32));
        assert_eq!(bitboard2.0, u64::pow(2, Square::A7 as u32));
        assert_eq!(bitboard3.0, u64::pow(2, Square::B8 as u32));
    }

    #[test]
    fn pop_unset_bit() {
        let mut bitboard1 = Bitboard(0);
        bitboard1.set_bit(Square::F1);
        bitboard1.pop_bit(Square::F1);
        bitboard1.pop_bit(Square::F1);
        let mut bitboard2 = Bitboard(0);
        bitboard2.pop_bit(Square::G2);

        assert_eq!(bitboard1.0, 0);
        assert_eq!(bitboard2.0, 0);
    }
}
