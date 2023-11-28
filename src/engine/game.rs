use super::{
    attack_tables,
    moves::{Move, MoveFlag, MoveList, MoveType},
};
use crate::uci::{FenError, InputError};
use num_derive::FromPrimitive;
use num_traits::{AsPrimitive, FromPrimitive, Unsigned};
use std::{
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, Shl, Shr, ShrAssign},
    str::FromStr,
    time::Instant,
};
use strum::{IntoEnumIterator, ParseError};
use strum_macros::{Display, EnumIter, EnumString};

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
        }
    }

    pub fn load_fen(&mut self, fen: &str) -> Result<(), InputError> {
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

        let fen = if fen == "startpos" {
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        } else {
            fen
        };

        let fen: Vec<&str> = fen.split_whitespace().collect();

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

        Ok(())
    }

    pub fn make_move(&mut self, mv: &Move, move_flag: MoveFlag) -> Result<(), InputError> {
        if move_flag == MoveFlag::Capture && mv.move_type() != MoveType::Capture {
            return Err(InputError::InvalidMoveFlag);
        }

        let mut game_clone = self.clone();

        let side = game_clone.side_to_move;
        let opponent_side = side.opponent_side();

        game_clone
            .mut_piece_bitboard(mv.piece(), side)
            .pop_bit(mv.source_square());

        match mv.promoted_piece() {
            Some(promoted_piece) => game_clone
                .mut_piece_bitboard(promoted_piece, side)
                .set_bit(mv.target_square()),
            None => game_clone
                .mut_piece_bitboard(mv.piece(), side)
                .set_bit(mv.target_square()),
        }

        let target_square_index = mv.target_square() as usize;

        match mv.move_type() {
            MoveType::Quiet => {}
            MoveType::Capture => {
                for piece in Piece::iter() {
                    game_clone
                        .mut_piece_bitboard(piece, opponent_side)
                        .pop_bit(mv.target_square());
                }
            }
            MoveType::DoublePawnPush => {
                let en_passant_square = match side {
                    Side::White => Square::from_usize(target_square_index + 8),
                    Side::Black => Square::from_usize(target_square_index - 8),
                }
                .unwrap();

                game_clone.en_passant_square = Some(en_passant_square);
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
            }
            MoveType::Castling => match side {
                Side::White => match mv.target_square() {
                    Square::C1 => {
                        game_clone
                            .mut_piece_bitboard(Piece::Rook, side)
                            .pop_bit(Square::A1);
                        game_clone
                            .mut_piece_bitboard(Piece::Rook, side)
                            .set_bit(Square::D1);
                    }
                    Square::G1 => {
                        game_clone
                            .mut_piece_bitboard(Piece::Rook, side)
                            .pop_bit(Square::H1);
                        game_clone
                            .mut_piece_bitboard(Piece::Rook, side)
                            .set_bit(Square::F1);
                    }
                    _ => {}
                },
                Side::Black => match mv.target_square() {
                    Square::C8 => {
                        game_clone
                            .mut_piece_bitboard(Piece::Rook, side)
                            .pop_bit(Square::A8);
                        game_clone
                            .mut_piece_bitboard(Piece::Rook, side)
                            .set_bit(Square::D8);
                    }
                    Square::G8 => {
                        game_clone
                            .mut_piece_bitboard(Piece::Rook, side)
                            .pop_bit(Square::H8);
                        game_clone
                            .mut_piece_bitboard(Piece::Rook, side)
                            .set_bit(Square::F8);
                    }
                    _ => {}
                },
            },
        }

        let king_square = game_clone
            .piece_bitboard(Piece::King, side)
            .get_lsb_square();

        if let Some(king_square) = king_square {
            let own_king_in_check = game_clone.is_square_attacked(opponent_side, king_square);

            if own_king_in_check {
                return Err(InputError::IllegalMove);
            }
        }

        if game_clone.castling_rights.0 != 0 {
            Self::update_castling_rights(&mut game_clone, mv);
        }

        if mv.move_type() != MoveType::DoublePawnPush {
            game_clone.en_passant_square = None;
        }

        game_clone.side_to_move = opponent_side;

        *self = game_clone;

        Ok(())
    }

    pub fn is_square_attacked(&self, attacking_side: Side, square: Square) -> bool {
        for piece in Piece::iter() {
            let piece_attacks_square = attack_tables::ATTACK_TABLES.attack_table(
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

    fn update_castling_rights(game_clone: &mut Game, mv: &Move) {
        let side = game_clone.side_to_move;

        match side {
            Side::White => match mv.source_square() {
                Square::A1 => game_clone
                    .castling_rights
                    .remove_castling_type(CastlingType::WhiteLong),
                Square::E1 => {
                    game_clone
                        .castling_rights
                        .remove_castling_type(CastlingType::WhiteShort);
                    game_clone
                        .castling_rights
                        .remove_castling_type(CastlingType::WhiteLong);
                }
                Square::H1 => game_clone
                    .castling_rights
                    .remove_castling_type(CastlingType::WhiteShort),
                _ => {}
            },
            Side::Black => match mv.source_square() {
                Square::A8 => game_clone
                    .castling_rights
                    .remove_castling_type(CastlingType::BlackLong),
                Square::E8 => {
                    game_clone
                        .castling_rights
                        .remove_castling_type(CastlingType::BlackShort);
                    game_clone
                        .castling_rights
                        .remove_castling_type(CastlingType::BlackLong);
                }
                Square::H8 => game_clone
                    .castling_rights
                    .remove_castling_type(CastlingType::BlackShort),
                _ => {}
            },
        }

        match side {
            Side::White => match mv.target_square() {
                Square::A8 => game_clone
                    .castling_rights
                    .remove_castling_type(CastlingType::BlackLong),
                Square::H8 => game_clone
                    .castling_rights
                    .remove_castling_type(CastlingType::BlackShort),
                _ => {}
            },
            Side::Black => match mv.target_square() {
                Square::A1 => game_clone
                    .castling_rights
                    .remove_castling_type(CastlingType::WhiteLong),
                Square::H1 => game_clone
                    .castling_rights
                    .remove_castling_type(CastlingType::WhiteShort),
                _ => {}
            },
        }
    }

    fn _print(&self) {
        for square in Square::iter() {
            if square.file() == 0 {
                print!("{:<4}", (64 - square as usize) / 8);
            }

            match self._piece_at_square(square) {
                Some((piece, side)) => print!("{:<2}", piece.to_char(Some(side))),
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
    }

    fn _piece_at_square(&self, square: Square) -> Option<(Piece, Side)> {
        for (bitboard, piece, side) in self._piece_bitboards() {
            if bitboard.bit_occupied(square) {
                return Some((piece, side));
            }
        }

        None
    }

    fn _piece_bitboards(&self) -> [(Bitboard, Piece, Side); 12] {
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

    pub fn bit_occupied(&self, square: Square) -> bool {
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

    pub fn count_bits(self) -> u32 {
        self.0.count_ones()
    }

    // lsb = least significant bit
    pub fn get_lsb_square(self) -> Option<Square> {
        if self.0 == 0 {
            return None;
        }

        Square::from_u32(self.0.trailing_zeros())
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

#[derive(Clone, Copy, Debug, Display, EnumIter, FromPrimitive, PartialEq)]
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

    pub fn to_char(self, side: Option<Side>) -> char {
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

#[derive(Clone, Copy, Debug, Display, PartialEq)]
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
}

#[derive(Clone, Copy, Debug, Display, EnumIter, EnumString, FromPrimitive, PartialEq)]
pub enum Square {
    A8,
    B8,
    C8,
    D8,
    E8,
    F8,
    G8,
    H8,
    A7,
    B7,
    C7,
    D7,
    E7,
    F7,
    G7,
    H7,
    A6,
    B6,
    C6,
    D6,
    E6,
    F6,
    G6,
    H6,
    A5,
    B5,
    C5,
    D5,
    E5,
    F5,
    G5,
    H5,
    A4,
    B4,
    C4,
    D4,
    E4,
    F4,
    G4,
    H4,
    A3,
    B3,
    C3,
    D3,
    E3,
    F3,
    G3,
    H3,
    A2,
    B2,
    C2,
    D2,
    E2,
    F2,
    G2,
    H2,
    A1,
    B1,
    C1,
    D1,
    E1,
    F1,
    G1,
    H1,
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

    pub fn to_lowercase_string(self) -> String {
        self.to_string().to_lowercase()
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

#[derive(Clone, Copy)]
pub enum CastlingType {
    WhiteShort = 0b1000,
    WhiteLong = 0b0100,
    BlackShort = 0b0010,
    BlackLong = 0b0001,
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

fn _perft_test(game: &mut Game, depth: u32) {
    let mut total_nodes = 0;
    let now = Instant::now();

    let move_list = MoveList::generate_moves(game);

    println!("Move   Nodes   ");

    for mv in move_list._move_list().iter().flatten() {
        let mut game_clone = game.clone();

        if game_clone.make_move(mv, MoveFlag::All).is_err() {
            continue;
        }

        let mut nodes = 0;

        _perft(&mut game_clone, &mut nodes, depth - 1);

        print!("{:<6}", mv._as_string());
        print!("{:^7}", nodes);
        println!();

        total_nodes += nodes;
    }

    println!();
    println!("Depth: {}", depth);
    println!("Nodes: {}", total_nodes);
    println!("Time taken: {:?}", now.elapsed());
}

fn _perft(game: &mut Game, nodes: &mut u64, depth: u32) {
    if depth == 0 {
        *nodes += 1;

        return;
    }

    let move_list = MoveList::generate_moves(game);

    for mv in move_list._move_list().iter().flatten() {
        let mut game_clone = game.clone();

        if game_clone.make_move(mv, MoveFlag::All).is_err() {
            continue;
        }

        _perft(&mut game_clone, nodes, depth - 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn perft_start_position() {
        let mut game = Game::initialise();

        game.load_fen("startpos").unwrap();

        let mut nodes = 0;

        _perft(&mut game, &mut nodes, 6);

        assert_eq!(nodes, 119_060_324);
    }

    #[test]
    #[ignore]
    fn perft_tricky_position() {
        let mut game = Game::initialise();

        game.load_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap();

        let mut nodes = 0;

        _perft(&mut game, &mut nodes, 5);

        assert_eq!(nodes, 193_690_690);
    }

    #[test]
    fn load_tricky_position() {
        let mut game = Game::initialise();

        game.load_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap();

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
    }

    #[test]
    fn load_killer_position() {
        let mut game = Game::initialise();

        game.load_fen("rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/3P3P/P1P1P3/RNBQKBNR w KQkq e6 0 1")
            .unwrap();

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
    }

    #[test]
    fn load_cmk_position() {
        let mut game = Game::initialise();

        game.load_fen("r2q1rk1/ppp2ppp/2n1bn2/2b1p3/3pP3/3P1NPP/PPP1NPB1/R1BQ1RK1 b - - 0 9")
            .unwrap();

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
    }

    #[test]
    fn set_bit() {
        let mut bitboard1 = Bitboard(0);
        let mut bitboard2 = Bitboard(0);
        let mut bitboard3 = Bitboard(0);

        bitboard1.set_bit(Square::H2);
        bitboard2.set_bit(Square::G6);
        bitboard3.set_bit(Square::B4);

        assert_eq!(bitboard1.0, u64::pow(2, Square::H2 as u32));
        assert_eq!(bitboard2.0, u64::pow(2, Square::G6 as u32));
        assert_eq!(bitboard3.0, u64::pow(2, Square::B4 as u32));
    }

    #[test]
    fn pop_bit() {
        let mut bitboard1 = Bitboard(0);
        let mut bitboard2 = Bitboard(0);
        let mut bitboard3 = Bitboard(0);

        bitboard1.set_bit(Square::G5);
        bitboard1.set_bit(Square::A8);
        bitboard1.pop_bit(Square::G5);

        bitboard2.set_bit(Square::C1);
        bitboard2.set_bit(Square::A7);
        bitboard2.pop_bit(Square::C1);

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
        let mut bitboard2 = Bitboard(0);

        bitboard1.set_bit(Square::F1);
        bitboard1.pop_bit(Square::F1);
        bitboard1.pop_bit(Square::F1);

        bitboard2.pop_bit(Square::G2);

        assert_eq!(bitboard1.0, 0);
        assert_eq!(bitboard2.0, 0);
    }
}
