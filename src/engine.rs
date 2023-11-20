mod attack_tables;
mod game;
mod moves;

use self::{game::Game, moves::MoveFlag};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{AsPrimitive, FromPrimitive, ToPrimitive, Unsigned};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, Shl, Shr, ShrAssign};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

const _TRICKY_POSITION: &str =
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
const CUSTOM_POSITION: &str =
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBqPPP/R3K2R w KQkq - 0 1";

pub fn position() {
    let mut game = Game::initialise(CUSTOM_POSITION);

    let moves = moves::generate_moves(&game);

    game.print();

    let _ = game.make_move(&moves[0], MoveFlag::All);
}

#[derive(Clone, Copy, PartialEq)]
pub struct Bitboard {
    bitboard: u64,
}

impl Bitboard {
    fn new(bitboard: u64) -> Self {
        Self { bitboard }
    }

    fn from_square(square: Square) -> Self {
        let mut bitboard = Self::new(0);

        bitboard.set_bit(square);

        bitboard
    }

    fn bit_occupied(&self, square: Square) -> bool {
        self.bitboard & (1 << square.to_usize().unwrap()) != 0
    }

    fn set_bit(&mut self, square: Square) {
        self.bitboard |= 1 << square.to_usize().unwrap();
    }

    fn pop_bit(&mut self, square: Square) {
        self.bitboard &= !(1 << square.to_usize().unwrap());
    }

    fn count_bits(&self) -> u32 {
        self.bitboard.count_ones()
    }

    // lsb = least significant bit
    fn get_lsb_index(&self) -> Option<usize> {
        if self.is_empty() {
            return None;
        }

        Some(self.bitboard.trailing_zeros() as usize)
    }

    fn is_empty(&self) -> bool {
        self.bitboard == 0
    }

    fn _print(&self) {
        for square in Square::iter() {
            if square.file() == 0 {
                print!("{}   ", ((64 - square.to_usize().unwrap()) / 8));
            }

            print!("{} ", if self.bit_occupied(square) { 1 } else { 0 });

            if square.file() == 7 {
                println!();
            }
        }

        println!();
        println!("    a b c d e f g h");
        println!();
        println!("    Bitboard decimal value: {}", self.bitboard);
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self::new(self.bitboard & rhs.bitboard)
    }
}

impl<T: Unsigned + AsPrimitive<u64>> BitAnd<T> for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: T) -> Self::Output {
        Self::new(self.bitboard & rhs.as_())
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = Self::new(self.bitboard & rhs.bitboard)
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::new(self.bitboard | rhs.bitboard)
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = Self::new(self.bitboard | rhs.bitboard)
    }
}

impl<T: Unsigned + AsPrimitive<u64>> BitOrAssign<T> for Bitboard {
    fn bitor_assign(&mut self, rhs: T) {
        *self = Self::new(self.bitboard | rhs.as_())
    }
}

impl Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::new(!self.bitboard)
    }
}

impl<T: Unsigned + AsPrimitive<u64>> PartialEq<T> for Bitboard {
    fn eq(&self, other: &T) -> bool {
        self.bitboard == other.as_()
    }
}

impl<T: Unsigned + AsPrimitive<u64>> Shl<T> for Bitboard {
    type Output = Self;

    fn shl(self, rhs: T) -> Self::Output {
        Self::new(self.bitboard << rhs.as_())
    }
}

impl<T: Unsigned + AsPrimitive<u64>> Shr<T> for Bitboard {
    type Output = Self;

    fn shr(self, rhs: T) -> Self::Output {
        Self::new(self.bitboard >> rhs.as_())
    }
}

impl<T: Unsigned + AsPrimitive<u64>> ShrAssign<T> for Bitboard {
    fn shr_assign(&mut self, rhs: T) {
        *self = Self::new(self.bitboard >> rhs.as_())
    }
}

#[derive(Clone, Copy, Debug, Display, EnumIter, FromPrimitive, PartialEq, ToPrimitive)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Piece {
    fn to_char(self, side: Side) -> char {
        match side {
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
        }
    }
}

#[derive(Clone, Copy, Debug, Display, PartialEq)]
pub enum Side {
    White,
    Black,
}

impl Side {
    fn opponent_side(&self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

#[derive(
    Clone, Copy, Debug, Display, EnumIter, EnumString, FromPrimitive, PartialEq, ToPrimitive,
)]
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
    fn from_rank_file(rank: usize, file: usize) -> Self {
        Self::from_usize(rank * 8 + file).unwrap()
    }

    fn rank(&self) -> usize {
        self.to_usize().unwrap() / 8
    }

    fn file(&self) -> usize {
        self.to_usize().unwrap() % 8
    }

    fn _to_lowercase_string(&self) -> String {
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

        bitboard1.set_bit(Square::H2);
        bitboard2.set_bit(Square::G6);
        bitboard3.set_bit(Square::B4);

        assert_eq!(
            bitboard1.bitboard,
            u64::pow(2, Square::H2.to_usize().unwrap() as u32)
        );
        assert_eq!(
            bitboard2.bitboard,
            u64::pow(2, Square::G6.to_usize().unwrap() as u32)
        );
        assert_eq!(
            bitboard3.bitboard,
            u64::pow(2, Square::B4.to_usize().unwrap() as u32)
        );
    }

    #[test]
    fn pop_bit() {
        let mut bitboard1 = Bitboard::new(0);
        let mut bitboard2 = Bitboard::new(0);
        let mut bitboard3 = Bitboard::new(0);

        bitboard1.set_bit(Square::G5);
        bitboard1.set_bit(Square::A8);
        bitboard1.pop_bit(Square::G5);

        bitboard2.set_bit(Square::C1);
        bitboard2.set_bit(Square::A7);
        bitboard2.pop_bit(Square::C1);

        bitboard3.set_bit(Square::C4);
        bitboard3.set_bit(Square::B8);
        bitboard3.pop_bit(Square::C4);

        assert_eq!(
            bitboard1.bitboard,
            u64::pow(2, Square::A8.to_usize().unwrap() as u32)
        );
        assert_eq!(
            bitboard2.bitboard,
            u64::pow(2, Square::A7.to_usize().unwrap() as u32)
        );
        assert_eq!(
            bitboard3.bitboard,
            u64::pow(2, Square::B8.to_usize().unwrap() as u32)
        );
    }

    #[test]
    fn pop_unset_bit() {
        let mut bitboard1 = Bitboard::new(0);
        let mut bitboard2 = Bitboard::new(0);

        bitboard1.set_bit(Square::F1);
        bitboard1.pop_bit(Square::F1);
        bitboard1.pop_bit(Square::F1);

        bitboard2.pop_bit(Square::G2);

        assert_eq!(bitboard1.bitboard, 0);
        assert_eq!(bitboard2.bitboard, 0);
    }
}
