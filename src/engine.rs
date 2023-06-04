mod attack_tables;

use self::attack_tables::{AttackTablesPub, LeaperAttackTables, SliderAttackTables};
use core::panic;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use std::str::FromStr;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

pub fn position() {
    // let empty_board = String::from("8/8/8/8/8/8/8/8 w - - ");
    // let start_position = String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 ");
    // let tricky_position =
    //     String::from("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1 ");
    // let killer_position =
    //     String::from("rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/3P3P/P1P1P3/RNBQKBNR w KQkq e6 0 1 ");
    // let cmk_position =
    //     String::from("r2q1rk1/ppp2ppp/2n1bn2/2b1p3/3pP3/3P1NPP/PPP1NPB1/R1BQ1RK1 b - - 0 9 ");

    let custom_position =
        String::from("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w Kkq e4 0 1 ");

    let leaper_attack_tables = LeaperAttackTables::initialise();
    let slider_attack_tables = SliderAttackTables::initialise();

    let board = Board::from_fen(custom_position);

    board.print();
}

struct Board {
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
    en_passant_square: Option<BoardSquare>,
    castling_rights: CastlingRights,
}

impl Board {
    fn initialise() -> Self {
        let mut white_pawns = Bitboard::new(0);
        let mut white_knights = Bitboard::new(0);
        let mut white_bishops = Bitboard::new(0);
        let mut white_rooks = Bitboard::new(0);
        let mut white_queens = Bitboard::new(0);
        let mut white_king = Bitboard::new(0);

        let mut black_pawns = Bitboard::new(0);
        let mut black_knights = Bitboard::new(0);
        let mut black_bishops = Bitboard::new(0);
        let mut black_rooks = Bitboard::new(0);
        let mut black_queens = Bitboard::new(0);
        let mut black_king = Bitboard::new(0);

        white_pawns.set_bit(&BoardSquare::A2);
        white_pawns.set_bit(&BoardSquare::B2);
        white_pawns.set_bit(&BoardSquare::C2);
        white_pawns.set_bit(&BoardSquare::D2);
        white_pawns.set_bit(&BoardSquare::E2);
        white_pawns.set_bit(&BoardSquare::F2);
        white_pawns.set_bit(&BoardSquare::G2);
        white_pawns.set_bit(&BoardSquare::H2);

        white_knights.set_bit(&BoardSquare::B1);
        white_knights.set_bit(&BoardSquare::G1);

        white_bishops.set_bit(&BoardSquare::C1);
        white_bishops.set_bit(&BoardSquare::F1);

        white_rooks.set_bit(&BoardSquare::A1);
        white_rooks.set_bit(&BoardSquare::H1);

        white_queens.set_bit(&BoardSquare::D1);

        white_king.set_bit(&BoardSquare::E1);

        black_pawns.set_bit(&BoardSquare::A7);
        black_pawns.set_bit(&BoardSquare::B7);
        black_pawns.set_bit(&BoardSquare::C7);
        black_pawns.set_bit(&BoardSquare::D7);
        black_pawns.set_bit(&BoardSquare::E7);
        black_pawns.set_bit(&BoardSquare::F7);
        black_pawns.set_bit(&BoardSquare::G7);
        black_pawns.set_bit(&BoardSquare::H7);

        black_knights.set_bit(&BoardSquare::B8);
        black_knights.set_bit(&BoardSquare::G8);

        black_bishops.set_bit(&BoardSquare::C8);
        black_bishops.set_bit(&BoardSquare::F8);

        black_rooks.set_bit(&BoardSquare::A8);
        black_rooks.set_bit(&BoardSquare::H8);

        black_queens.set_bit(&BoardSquare::D8);

        black_king.set_bit(&BoardSquare::E8);

        Self {
            white_pawns,
            white_knights,
            white_bishops,
            white_rooks,
            white_queens,
            white_king,
            black_pawns,
            black_knights,
            black_bishops,
            black_rooks,
            black_queens,
            black_king,
            side_to_move: Side::White,
            en_passant_square: None,
            castling_rights: CastlingRights::initialise("KQkq"),
        }
    }

    fn from_fen(fen: String) -> Self {
        let mut white_pawns = Bitboard::new(0);
        let mut white_knights = Bitboard::new(0);
        let mut white_bishops = Bitboard::new(0);
        let mut white_rooks = Bitboard::new(0);
        let mut white_queens = Bitboard::new(0);
        let mut white_king = Bitboard::new(0);

        let mut black_pawns = Bitboard::new(0);
        let mut black_knights = Bitboard::new(0);
        let mut black_bishops = Bitboard::new(0);
        let mut black_rooks = Bitboard::new(0);
        let mut black_queens = Bitboard::new(0);
        let mut black_king = Bitboard::new(0);

        let fen: Vec<&str> = fen.split_whitespace().collect();

        let mut square_index = 0;

        fen[0].chars().for_each(|character| match character {
            'P' => {
                white_pawns.set_bit(&BoardSquare::new_from_index(square_index));
                square_index += 1;
            }
            'N' => {
                white_knights.set_bit(&BoardSquare::new_from_index(square_index));
                square_index += 1;
            }
            'B' => {
                white_bishops.set_bit(&BoardSquare::new_from_index(square_index));
                square_index += 1;
            }
            'R' => {
                white_rooks.set_bit(&BoardSquare::new_from_index(square_index));
                square_index += 1;
            }
            'Q' => {
                white_queens.set_bit(&BoardSquare::new_from_index(square_index));
                square_index += 1;
            }
            'K' => {
                white_king.set_bit(&BoardSquare::new_from_index(square_index));
                square_index += 1;
            }
            'p' => {
                black_pawns.set_bit(&BoardSquare::new_from_index(square_index));
                square_index += 1;
            }
            'n' => {
                black_knights.set_bit(&BoardSquare::new_from_index(square_index));
                square_index += 1;
            }
            'b' => {
                black_bishops.set_bit(&BoardSquare::new_from_index(square_index));
                square_index += 1;
            }
            'r' => {
                black_rooks.set_bit(&BoardSquare::new_from_index(square_index));
                square_index += 1;
            }
            'q' => {
                black_queens.set_bit(&BoardSquare::new_from_index(square_index));
                square_index += 1;
            }
            'k' => {
                black_king.set_bit(&BoardSquare::new_from_index(square_index));
                square_index += 1;
            }
            '/' => {}
            '0'..='9' => square_index += character as usize - '0' as usize,
            _ => panic!("Attempted to use invalid character in FEN string"),
        });

        Self {
            white_pawns,
            white_knights,
            white_bishops,
            white_rooks,
            white_queens,
            white_king,
            black_pawns,
            black_knights,
            black_bishops,
            black_rooks,
            black_queens,
            black_king,
            side_to_move: if fen[1] == "w" {
                Side::White
            } else {
                Side::Black
            },
            en_passant_square: if fen[3] == "-" {
                None
            } else {
                Some(BoardSquare::new_from_string(fen[3]))
            },
            castling_rights: CastlingRights::initialise(fen[2]),
        }
    }

    fn print(&self) {
        BoardSquare::iter().for_each(|square| {
            if square.file() == 0 {
                print!("{}   ", (64 - square.as_usize()) / 8);
            }

            match self.piece_at_square(&square) {
                Some((piece, side)) => print!("{} ", Self::get_piece_character(&piece, &side)),
                None => print!(". "),
            }

            if square.file() == 7 {
                println!();
            }
        });

        println!();
        println!("    a b c d e f g h");
        println!();
        println!("Side to move: {:?}", self.side_to_move);
        println!("En passant square: {:?}", self.en_passant_square);
        println!("Castling rights: {}", self.castling_rights.as_string());
    }

    fn piece_bitboards(&self) -> [(Bitboard, Piece, Side); 12] {
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

    fn piece_at_square(&self, square: &BoardSquare) -> Option<(Piece, Side)> {
        for bitboard in self.piece_bitboards() {
            if bitboard.0.bit_occupied(square) {
                return Some((bitboard.1, bitboard.2));
            }
        }

        None
    }

    fn get_piece_character(piece: &Piece, side: &Side) -> char {
        match side {
            Side::White => match piece {
                Piece::Pawn => 'P',
                Piece::Knight => 'N',
                Piece::Bishop => 'B',
                Piece::Rook => 'R',
                Piece::Queen => 'Q',
                Piece::King => 'K',
            },
            Side::Black => match piece {
                Piece::Pawn => 'p',
                Piece::Knight => 'n',
                Piece::Bishop => 'b',
                Piece::Rook => 'r',
                Piece::Queen => 'q',
                Piece::King => 'k',
            },

            Side::Either => panic!("Attempted to get piece character without specifying side"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Bitboard {
    bitboard: u64,
}

impl Bitboard {
    fn new(bitboard: u64) -> Self {
        Bitboard { bitboard }
    }

    fn bit_occupied(&self, square: &BoardSquare) -> bool {
        self.bitboard & (1 << square.as_usize()) != 0
    }

    fn set_bit(&mut self, square: &BoardSquare) {
        self.bitboard |= 1 << square.as_usize();
    }

    fn pop_bit(&mut self, square: &BoardSquare) {
        self.bitboard &= !(1 << square.as_usize());
    }

    fn count_bits(&self) -> u32 {
        self.bitboard.count_ones()
    }

    // ls1b = least significant 1st bit
    fn get_ls1b_index(&self) -> Option<usize> {
        if self.is_empty() {
            return None;
        }

        Some(self.bitboard.trailing_zeros() as usize)
    }

    fn is_empty(&self) -> bool {
        self.bitboard == 0
    }

    fn print(&self) {
        BoardSquare::iter().for_each(|square| {
            if square.file() == 0 {
                print!("{}   ", (64 - square.as_usize() / 8));
            }

            print!("{} ", if self.bit_occupied(&square) { 1 } else { 0 });

            if square.file() == 7 {
                println!();
            }
        });

        println!();
        println!("    a b c d e f g h");
        println!();
        println!("    Bitboard decimal value: {}", self.bitboard);
    }
}

trait EnumToInt: ToPrimitive {
    fn as_usize(&self) -> usize {
        match self.to_usize() {
            Some(value) => value,
            None => panic!("Failed to convert enum to usize type"),
        }
    }

    fn as_u8(&self) -> u8 {
        match self.to_u8() {
            Some(value) => value,
            None => panic!("Failed to convert enum to u32 type"),
        }
    }
}

struct CastlingRights {
    castling_rights: u8,
}

impl CastlingRights {
    fn initialise(castling_rights_string: &str) -> Self {
        if castling_rights_string == "-" {
            return Self { castling_rights: 0 };
        };

        let mut castling_rights = 0;

        castling_rights_string
            .chars()
            .for_each(|character| match character {
                'K' => castling_rights |= CastlingTypes::WhiteShort.as_u8(),
                'Q' => castling_rights |= CastlingTypes::WhiteLong.as_u8(),
                'k' => castling_rights |= CastlingTypes::BlackShort.as_u8(),
                'q' => castling_rights |= CastlingTypes::BlackLong.as_u8(),
                _ => panic!("Invalid character used when attempting to initialise castling rights"),
            });

        Self { castling_rights }
    }

    fn as_string(&self) -> String {
        let mut castling_rights_string = String::new();

        if self.castling_rights & CastlingTypes::WhiteShort.as_u8() != 0 {
            castling_rights_string.push('K');
        }

        if self.castling_rights & CastlingTypes::WhiteLong.as_u8() != 0 {
            castling_rights_string.push('Q');
        }

        if self.castling_rights & CastlingTypes::BlackShort.as_u8() != 0 {
            castling_rights_string.push('k');
        }

        if self.castling_rights & CastlingTypes::BlackLong.as_u8() != 0 {
            castling_rights_string.push('q');
        }

        castling_rights_string
    }
}

#[derive(Debug)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug)]
pub enum Side {
    White,
    Black,
    Either,
}

#[derive(Debug, Display, EnumIter, EnumString, FromPrimitive, ToPrimitive)]
pub enum BoardSquare {
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

impl EnumToInt for BoardSquare {}

impl BoardSquare {
    fn new_from_index(index: usize) -> Self {
        let square_option = Self::from_usize(index);

        match square_option {
            None => panic!("Attempted to convert invalid index into board square"),
            Some(square) => square,
        }
    }

    fn new_from_string(square: &str) -> Self {
        match BoardSquare::from_str(&square.to_uppercase()) {
            Ok(square) => square,
            Err(_) => panic!("Attempted to convert invalid string slice into board square"),
        }
    }

    fn rank(&self) -> usize {
        self.as_usize() / 8
    }

    fn file(&self) -> usize {
        self.as_usize() % 8
    }

    fn to_lowercase_string(&self) -> String {
        self.to_string().to_lowercase()
    }
}

#[derive(ToPrimitive)]
enum CastlingTypes {
    WhiteShort = 0b1000,
    WhiteLong = 0b0100,
    BlackShort = 0b0010,
    BlackLong = 0b0001,
}

impl EnumToInt for CastlingTypes {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_bit() {
        let mut bitboard1 = Bitboard::new(0);
        let mut bitboard2 = Bitboard::new(0);
        let mut bitboard3 = Bitboard::new(0);

        bitboard1.set_bit(&BoardSquare::H2);
        bitboard2.set_bit(&BoardSquare::G6);
        bitboard3.set_bit(&BoardSquare::B4);

        assert_eq!(
            bitboard1.bitboard,
            u64::pow(2, BoardSquare::H2.as_usize() as u32)
        );
        assert_eq!(
            bitboard2.bitboard,
            u64::pow(2, BoardSquare::G6.as_usize() as u32)
        );
        assert_eq!(
            bitboard3.bitboard,
            u64::pow(2, BoardSquare::B4.as_usize() as u32)
        );
    }

    #[test]
    fn pop_bit() {
        let mut bitboard1 = Bitboard::new(0);
        let mut bitboard2 = Bitboard::new(0);
        let mut bitboard3 = Bitboard::new(0);

        bitboard1.set_bit(&BoardSquare::G5);
        bitboard1.set_bit(&BoardSquare::A8);
        bitboard1.pop_bit(&BoardSquare::G5);

        bitboard2.set_bit(&BoardSquare::C1);
        bitboard2.set_bit(&BoardSquare::A7);
        bitboard2.pop_bit(&BoardSquare::C1);

        bitboard3.set_bit(&BoardSquare::C4);
        bitboard3.set_bit(&BoardSquare::B8);
        bitboard3.pop_bit(&BoardSquare::C4);

        assert_eq!(
            bitboard1.bitboard,
            u64::pow(2, BoardSquare::A8.as_usize() as u32)
        );
        assert_eq!(
            bitboard2.bitboard,
            u64::pow(2, BoardSquare::A7.as_usize() as u32)
        );
        assert_eq!(
            bitboard3.bitboard,
            u64::pow(2, BoardSquare::B8.as_usize() as u32)
        );
    }

    #[test]
    fn pop_unset_bit() {
        let mut bitboard1 = Bitboard::new(0);
        let mut bitboard2 = Bitboard::new(0);

        bitboard1.set_bit(&BoardSquare::F1);
        bitboard1.pop_bit(&BoardSquare::F1);
        bitboard1.pop_bit(&BoardSquare::F1);

        bitboard2.pop_bit(&BoardSquare::G2);

        assert_eq!(bitboard1.bitboard, 0);
        assert_eq!(bitboard2.bitboard, 0);
    }
}
