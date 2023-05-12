//! # Engine
//!
//! A library that calculates the best move based on the current board position

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

    attack_tables_rook
        .attack_tables
        .iter()
        .for_each(|bitboard| bitboard.print());
}

#[derive(Clone, Copy)]
struct Bitboard {
    bitboard: u64,
}

impl Bitboard {
    fn new(bitboard: u64) -> Self {
        Bitboard { bitboard }
    }

    fn print(&self) {
        BoardSquare::iter().for_each(|square| {
            if (square as u64) % 8 == 0 {
                print!("{}   ", (64 - square as u64) / 8);
            }

            print!("{} ", if self.get_bit(square) { 1 } else { 0 });

            if (square as u64) % 8 == 7 {
                println!("");
            }
        });

        println!("");
        println!("    a b c d e f g h");
        println!("");
        println!("    Bitboard decimal value: {}", self.bitboard);
    }

    fn get_bit(&self, square: BoardSquare) -> bool {
        self.bitboard & (1 << square as u64) != 0
    }

    fn set_bit(&mut self, square: BoardSquare) {
        self.bitboard |= 1 << square as u64;
    }

    fn pop_bit(&mut self, square: BoardSquare) {
        self.bitboard &= !(1 << square as u64);
    }
}

struct AttackTables {
    attack_tables: [Bitboard; 64],
}

impl AttackTables {
    fn new(piece: Piece, side: Side) -> Self {
        if matches!(piece, Piece::Pawn) && matches!(side, Side::Either) {
            panic!("Attempted to instantiate pawn attack table with side == Side::Either");
        }

        Self {
            attack_tables: Self::generate_attack_tables(piece, side),
        }
    }

    fn generate_attack_tables(piece: Piece, side: Side) -> [Bitboard; 64] {
        // Bitboards with all values initialised to 1, except for the file(s) indicated
        // Used to prevent incorrect attack table generation for pieces on / near edge files
        let file_a_zeroed = Bitboard::new(18374403900871474942);
        let file_h_zeroed = Bitboard::new(9187201950435737471);
        let file_ab_zeroed = Bitboard::new(18229723555195321596);
        let file_gh_zeroed = Bitboard::new(4557430888798830399);

        let mut attack_tables: [Bitboard; 64] = [Bitboard::new(0); 64];

        BoardSquare::iter().for_each(|square| {
            let mut bitboard = Bitboard::new(0);
            let mut attack_table = Bitboard::new(0);

            bitboard.set_bit(square);

            match piece {
                Piece::Pawn => {
                    if matches!(side, Side::White) {
                        attack_table.bitboard |= (bitboard.bitboard >> 7) & file_a_zeroed.bitboard;
                        attack_table.bitboard |= (bitboard.bitboard >> 9) & file_h_zeroed.bitboard;
                    } else {
                        attack_table.bitboard |= (bitboard.bitboard << 7) & file_h_zeroed.bitboard;
                        attack_table.bitboard |= (bitboard.bitboard << 9) & file_a_zeroed.bitboard;
                    }
                }
                Piece::Knight => {
                    attack_table.bitboard |= (bitboard.bitboard >> 6) & file_ab_zeroed.bitboard;
                    attack_table.bitboard |= (bitboard.bitboard >> 10) & file_gh_zeroed.bitboard;
                    attack_table.bitboard |= (bitboard.bitboard >> 15) & file_a_zeroed.bitboard;
                    attack_table.bitboard |= (bitboard.bitboard >> 17) & file_h_zeroed.bitboard;
                    attack_table.bitboard |= (bitboard.bitboard << 6) & file_gh_zeroed.bitboard;
                    attack_table.bitboard |= (bitboard.bitboard << 10) & file_ab_zeroed.bitboard;
                    attack_table.bitboard |= (bitboard.bitboard << 15) & file_h_zeroed.bitboard;
                    attack_table.bitboard |= (bitboard.bitboard << 17) & file_a_zeroed.bitboard;
                }
                Piece::King => {
                    attack_table.bitboard |= (bitboard.bitboard >> 1) & file_h_zeroed.bitboard;
                    attack_table.bitboard |= (bitboard.bitboard >> 7) & file_a_zeroed.bitboard;
                    attack_table.bitboard |= bitboard.bitboard >> 8;
                    attack_table.bitboard |= (bitboard.bitboard >> 9) & file_h_zeroed.bitboard;
                    attack_table.bitboard |= (bitboard.bitboard << 1) & file_a_zeroed.bitboard;
                    attack_table.bitboard |= (bitboard.bitboard << 7) & file_h_zeroed.bitboard;
                    attack_table.bitboard |= bitboard.bitboard << 8;
                    attack_table.bitboard |= (bitboard.bitboard << 9) & file_a_zeroed.bitboard;
                }
                _ => attack_table.bitboard = Self::generate_slider_attack_table(piece, square),
            }

            attack_tables[square as usize].bitboard = attack_table.bitboard;
        });

        attack_tables
    }

    fn generate_slider_attack_table(piece: Piece, square: BoardSquare) -> u64 {
        let mut attack_table = Bitboard::new(0);

        let target_rank = (square as usize) / 8;
        let target_file = (square as usize) % 8;

        // Cardinal occupancy
        if matches!(piece, Piece::Rook) || matches!(piece, Piece::Queen) {
            for rank in (target_rank + 1)..7 {
                attack_table.bitboard |= 1 << rank * 8 + target_file;
            }
            for rank in 1..target_rank {
                attack_table.bitboard |= 1 << rank * 8 + target_file;
            }
            for file in (target_file + 1)..7 {
                attack_table.bitboard |= 1 << target_rank * 8 + file;
            }
            for file in 1..target_file {
                attack_table.bitboard |= 1 << target_rank * 8 + file;
            }
        }

        // Diagonal occupancy
        if matches!(piece, Piece::Bishop) || matches!(piece, Piece::Queen) {
            for (rank, file) in ((target_rank + 1)..7).zip((target_file + 1)..7) {
                attack_table.bitboard |= 1 << (rank * 8 + file);
            }
            for (rank, file) in ((1..target_rank).rev()).zip((target_file + 1)..7) {
                attack_table.bitboard |= 1 << (rank * 8 + file);
            }
            for (rank, file) in ((target_rank + 1)..7).zip((1..target_file).rev()) {
                attack_table.bitboard |= 1 << (rank * 8 + file);
            }
            for (rank, file) in ((1..target_rank).rev()).zip((1..target_file).rev()) {
                attack_table.bitboard |= 1 << (rank * 8 + file);
            }
        }

        attack_table.bitboard
    }
}

#[derive(Clone, Copy)]
enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Clone, Copy)]
enum Side {
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
