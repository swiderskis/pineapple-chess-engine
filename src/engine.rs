mod attack_tables;

use self::attack_tables::{AttackTablesPub, LeaperAttackTables, SliderAttackTables};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

pub fn position() {
    let leaper_attack_tables = LeaperAttackTables::initialise();
    let slider_attack_tables = SliderAttackTables::initialise();

    let board = Board::initialise();

    board.white_pawns.print();
    board.white_knights.print();
    board.white_bishops.print();
    board.white_rooks.print();
    board.white_queens.print();
    board.white_king.print();

    board.black_pawns.print();
    board.black_knights.print();
    board.black_bishops.print();
    board.black_rooks.print();
    board.black_queens.print();
    board.black_king.print();
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

        for square in (BoardSquare::A2.enumeration())..=(BoardSquare::H2.enumeration()) {
            white_pawns.set_bit(&BoardSquare::new_from_index(square));
        }

        white_knights.set_bit(&BoardSquare::B1);
        white_knights.set_bit(&BoardSquare::G1);

        white_bishops.set_bit(&BoardSquare::C1);
        white_bishops.set_bit(&BoardSquare::F1);

        white_rooks.set_bit(&BoardSquare::A1);
        white_rooks.set_bit(&BoardSquare::H1);

        white_queens.set_bit(&BoardSquare::D1);

        white_king.set_bit(&BoardSquare::E1);

        for square in (BoardSquare::A7.enumeration())..=(BoardSquare::H7.enumeration()) {
            black_pawns.set_bit(&BoardSquare::new_from_index(square));
        }

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

    fn get_bit(&self, square: &BoardSquare) -> bool {
        self.bitboard & (1 << square.enumeration()) != 0
    }

    fn set_bit(&mut self, square: &BoardSquare) {
        self.bitboard |= 1 << square.enumeration();
    }

    fn pop_bit(&mut self, square: &BoardSquare) {
        self.bitboard &= !(1 << square.enumeration());
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
                print!("{}   ", (64 - square.enumeration()) / 8);
            }

            print!("{} ", if self.get_bit(&square) { 1 } else { 0 });

            if square.file() == 7 {
                println!("");
            }
        });

        println!("");
        println!("    a b c d e f g h");
        println!("");
        println!("    Bitboard decimal value: {}", self.bitboard);
    }
}

pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

pub enum Side {
    White,
    Black,
    Either,
}

#[derive(Clone, Display, EnumIter, FromPrimitive)]
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

impl BoardSquare {
    fn new_from_index(index: usize) -> Self {
        let square_option = Self::from_usize(index);

        match square_option {
            None => panic!("Attempted to convert invalid index into board square"),
            Some(square) => square,
        }
    }

    fn enumeration(&self) -> usize {
        self.clone() as usize
    }

    fn rank(&self) -> usize {
        self.enumeration() / 8
    }

    fn file(&self) -> usize {
        self.enumeration() % 8
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

        bitboard1.set_bit(&BoardSquare::H2);
        bitboard2.set_bit(&BoardSquare::G6);
        bitboard3.set_bit(&BoardSquare::B4);

        assert_eq!(
            bitboard1.bitboard,
            u64::pow(2, BoardSquare::H2.enumeration() as u32)
        );
        assert_eq!(
            bitboard2.bitboard,
            u64::pow(2, BoardSquare::G6.enumeration() as u32)
        );
        assert_eq!(
            bitboard3.bitboard,
            u64::pow(2, BoardSquare::B4.enumeration() as u32)
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
            u64::pow(2, BoardSquare::A8.enumeration() as u32)
        );
        assert_eq!(
            bitboard2.bitboard,
            u64::pow(2, BoardSquare::A7.enumeration() as u32)
        );
        assert_eq!(
            bitboard3.bitboard,
            u64::pow(2, BoardSquare::B8.enumeration() as u32)
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
