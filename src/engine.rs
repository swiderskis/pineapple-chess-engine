mod attack_tables;
mod game;
mod generate_moves;

use self::game::Game;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use std::str::FromStr;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

pub fn position() {
    let game = Game::initialise("startpos");

    let moves = generate_moves::generate_moves(&game);

    moves.print_move_list();
}

#[derive(Clone, Copy)]
pub struct Bitboard {
    bitboard: u64,
}

impl Bitboard {
    fn new(bitboard: u64) -> Self {
        Bitboard { bitboard }
    }

    fn from_square(square: &Square) -> Self {
        let mut bitboard = Bitboard::new(0);

        bitboard.set_bit(square);

        bitboard
    }

    fn bit_occupied(&self, square: &Square) -> bool {
        self.bitboard & (1 << square.as_usize()) != 0
    }

    fn set_bit(&mut self, square: &Square) {
        self.bitboard |= 1 << square.as_usize();
    }

    fn pop_bit(&mut self, square: &Square) {
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

    fn _print(&self) {
        Square::iter().for_each(|square| {
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

    fn as_u32(&self) -> u32 {
        match self.to_u32() {
            Some(value) => value,
            None => panic!("Failed to convert enum to u32 type"),
        }
    }

    fn as_u8(&self) -> u8 {
        match self.to_u8() {
            Some(value) => value,
            None => panic!("Failed to convert enum to u8 type"),
        }
    }
}

#[derive(Debug, Display, FromPrimitive, ToPrimitive)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Piece {
    fn new_from_u32(value: u32) -> Self {
        let piece_option = Self::from_u32(value);

        match piece_option {
            Some(piece) => piece,
            None => panic!("Attempted to convert invalid index into piece"),
        }
    }

    fn to_char(&self, side: Option<&Side>) -> char {
        match side {
            Some(Side::White) => match self {
                Self::Pawn => 'P',
                Self::Knight => 'N',
                Self::Bishop => 'B',
                Self::Rook => 'R',
                Self::Queen => 'Q',
                Self::King => 'K',
            },
            _ => match self {
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

impl EnumToInt for Piece {}

#[derive(Debug)]
pub enum Side {
    White,
    Black,
}

impl Side {
    fn opponent_side(&self) -> Side {
        match self {
            Self::White => Side::Black,
            Self::Black => Side::White,
        }
    }
}

#[derive(Debug, Display, EnumIter, EnumString, FromPrimitive, ToPrimitive)]
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

impl EnumToInt for Square {}

impl Square {
    fn new_from_index(index: usize) -> Self {
        let square_option = Self::from_usize(index);

        match square_option {
            Some(square) => square,
            None => panic!("Attempted to convert invalid index into board square"),
        }
    }

    fn new_from_string(square: &str) -> Self {
        match Square::from_str(&square.to_uppercase()) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_bit() {
        let mut bitboard1 = Bitboard::new(0);
        let mut bitboard2 = Bitboard::new(0);
        let mut bitboard3 = Bitboard::new(0);

        bitboard1.set_bit(&Square::H2);
        bitboard2.set_bit(&Square::G6);
        bitboard3.set_bit(&Square::B4);

        assert_eq!(
            bitboard1.bitboard,
            u64::pow(2, Square::H2.as_usize() as u32)
        );
        assert_eq!(
            bitboard2.bitboard,
            u64::pow(2, Square::G6.as_usize() as u32)
        );
        assert_eq!(
            bitboard3.bitboard,
            u64::pow(2, Square::B4.as_usize() as u32)
        );
    }

    #[test]
    fn pop_bit() {
        let mut bitboard1 = Bitboard::new(0);
        let mut bitboard2 = Bitboard::new(0);
        let mut bitboard3 = Bitboard::new(0);

        bitboard1.set_bit(&Square::G5);
        bitboard1.set_bit(&Square::A8);
        bitboard1.pop_bit(&Square::G5);

        bitboard2.set_bit(&Square::C1);
        bitboard2.set_bit(&Square::A7);
        bitboard2.pop_bit(&Square::C1);

        bitboard3.set_bit(&Square::C4);
        bitboard3.set_bit(&Square::B8);
        bitboard3.pop_bit(&Square::C4);

        assert_eq!(
            bitboard1.bitboard,
            u64::pow(2, Square::A8.as_usize() as u32)
        );
        assert_eq!(
            bitboard2.bitboard,
            u64::pow(2, Square::A7.as_usize() as u32)
        );
        assert_eq!(
            bitboard3.bitboard,
            u64::pow(2, Square::B8.as_usize() as u32)
        );
    }

    #[test]
    fn pop_unset_bit() {
        let mut bitboard1 = Bitboard::new(0);
        let mut bitboard2 = Bitboard::new(0);

        bitboard1.set_bit(&Square::F1);
        bitboard1.pop_bit(&Square::F1);
        bitboard1.pop_bit(&Square::F1);

        bitboard2.pop_bit(&Square::G2);

        assert_eq!(bitboard1.bitboard, 0);
        assert_eq!(bitboard2.bitboard, 0);
    }
}
