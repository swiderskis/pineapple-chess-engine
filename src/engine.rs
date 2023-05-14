mod attack_tables;

use attack_tables::AttackTables;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub fn position() {
    let attack_tables_white_pawn = AttackTables::new(Piece::Pawn, Side::White);
    let attack_tables_black_pawn = AttackTables::new(Piece::Pawn, Side::Black);
    let attack_tables_knight = AttackTables::new(Piece::Knight, Side::Either);
    let attack_tables_bishop = AttackTables::new(Piece::Bishop, Side::Either);
    let attack_tables_rook = AttackTables::new(Piece::Rook, Side::Either);
    let attack_tables_queen = AttackTables::new(Piece::Queen, Side::Either);
    let attack_tables_king = AttackTables::new(Piece::King, Side::Either);
}

#[derive(Clone, Copy)]
pub struct Bitboard {
    bitboard: u64,
}

impl Bitboard {
    fn new(bitboard: u64) -> Self {
        Bitboard { bitboard }
    }

    fn print(&self) {
        BoardSquare::iter().for_each(|square| {
            if (square.enumeration()) % 8 == 0 {
                print!("{}   ", (64 - square.enumeration()) / 8);
            }

            print!("{} ", if self.get_bit(square) { 1 } else { 0 });

            if square.file() == 7 {
                println!("");
            }
        });

        println!("");
        println!("    a b c d e f g h");
        println!("");
        println!("    Bitboard decimal value: {}", self.bitboard);
    }

    fn get_bit(&self, square: BoardSquare) -> bool {
        self.bitboard & (1 << square.enumeration()) != 0
    }

    fn set_bit(&mut self, square: BoardSquare) {
        self.bitboard |= 1 << square.enumeration();
    }

    fn pop_bit(&mut self, square: BoardSquare) {
        self.bitboard &= !(1 << square.enumeration());
    }
}

#[derive(Clone, Copy)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Clone, Copy)]
pub enum Side {
    White,
    Black,
    Either,
}

#[derive(Clone, Copy, EnumIter)]
enum BoardSquare {
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
    fn enumeration(self) -> usize {
        self as usize
    }

    fn rank(self) -> usize {
        self.enumeration() / 8
    }

    fn file(self) -> usize {
        self.enumeration() % 8
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

        bitboard1.set_bit(BoardSquare::H2);
        bitboard2.set_bit(BoardSquare::G6);
        bitboard3.set_bit(BoardSquare::B4);

        assert_eq!(bitboard1.bitboard, u64::pow(2, 55));
        assert_eq!(bitboard2.bitboard, u64::pow(2, 22));
        assert_eq!(bitboard3.bitboard, u64::pow(2, 33));
    }

    #[test]
    fn pop_bit() {
        let mut bitboard1 = Bitboard::new(0);
        let mut bitboard2 = Bitboard::new(0);
        let mut bitboard3 = Bitboard::new(0);

        bitboard1.set_bit(BoardSquare::G5);
        bitboard1.set_bit(BoardSquare::A8);
        bitboard1.pop_bit(BoardSquare::G5);

        bitboard2.set_bit(BoardSquare::C1);
        bitboard2.set_bit(BoardSquare::A7);
        bitboard2.pop_bit(BoardSquare::C1);

        bitboard3.set_bit(BoardSquare::C4);
        bitboard3.set_bit(BoardSquare::B8);
        bitboard3.pop_bit(BoardSquare::C4);

        assert_eq!(bitboard1.bitboard, u64::pow(2, 0));
        assert_eq!(bitboard2.bitboard, u64::pow(2, 8));
        assert_eq!(bitboard3.bitboard, u64::pow(2, 1));
    }

    #[test]
    fn pop_unset_bit() {
        let mut bitboard1 = Bitboard::new(0);
        let mut bitboard2 = Bitboard::new(0);

        bitboard1.set_bit(BoardSquare::F1);
        bitboard1.pop_bit(BoardSquare::F1);
        bitboard1.pop_bit(BoardSquare::F1);

        bitboard2.pop_bit(BoardSquare::G2);

        assert_eq!(bitboard1.bitboard, 0);
        assert_eq!(bitboard2.bitboard, 0);
    }
}
