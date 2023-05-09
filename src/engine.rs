//! # Engine
//!
//! A library that calculates the best move based on the current board position

/// Sets up the current board position, either from a given FEN string or the initial board position
pub fn position() {
    let mut bitboard = Bitboard::new();

    bitboard.set_bit(BoardSquare::H1);

    bitboard.print_bitboard();
}

struct Bitboard {
    bitboard: u64,
}

impl Bitboard {
    /// Bitboard constructor
    fn new() -> Self {
        Bitboard { bitboard: 0 }
    }

    /// Prints the current bitboard state
    fn print_bitboard(&self) {
        for rank in 0..8 {
            for file in 0..8 {
                let square = rank * 8 + file;

                if file == 0 {
                    print!("{}   ", 8 - rank);
                }

                print!("{} ", if self.get_bit(square) { 1 } else { 0 });
            }

            println!("");
        }

        println!("");
        println!("    a b c d e f g h");
        println!("");
        println!("    Bitboard decimal value: {}", self.bitboard);
    }

    fn get_bit(&self, square: i32) -> bool {
        self.bitboard & (1 << square) != 0
    }

    fn set_bit(&mut self, square: BoardSquare) {
        self.bitboard |= 1 << square as u64;
    }

    fn pop_bit(&mut self, square: BoardSquare) {
        self.bitboard &= !(1 << square as u64);
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_bit() {
        let mut bitboard1 = Bitboard::new();
        let mut bitboard2 = Bitboard::new();
        let mut bitboard3 = Bitboard::new();

        bitboard1.set_bit(BoardSquare::H2);
        bitboard2.set_bit(BoardSquare::G6);
        bitboard3.set_bit(BoardSquare::B4);

        assert_eq!(bitboard1.bitboard, u64::pow(2, 55));
        assert_eq!(bitboard2.bitboard, u64::pow(2, 22));
        assert_eq!(bitboard3.bitboard, u64::pow(2, 33));
    }

    #[test]
    fn pop_bit() {
        let mut bitboard1 = Bitboard::new();
        let mut bitboard2 = Bitboard::new();
        let mut bitboard3 = Bitboard::new();

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
        let mut bitboard1 = Bitboard::new();
        let mut bitboard2 = Bitboard::new();

        bitboard1.set_bit(BoardSquare::F1);
        bitboard1.pop_bit(BoardSquare::F1);
        bitboard1.pop_bit(BoardSquare::F1);

        bitboard2.pop_bit(BoardSquare::G2);

        assert_eq!(bitboard1.bitboard, 0);
        assert_eq!(bitboard2.bitboard, 0);
    }
}
