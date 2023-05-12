use super::{Bitboard, BoardSquare, Piece, Side};
use strum::IntoEnumIterator;

pub struct AttackTables {
    attack_tables: [Bitboard; 64],
}

impl AttackTables {
    pub fn new(piece: Piece, side: Side) -> Self {
        if matches!(piece, Piece::Pawn) && matches!(side, Side::Either) {
            panic!("Attempted to instantiate pawn attack table with side == Side::Either");
        }

        Self {
            attack_tables: Self::generate_attack_tables(piece, side),
        }
    }

    pub fn attack_tables(&self) -> [Bitboard; 64] {
        self.attack_tables
    }

    pub fn attack_table(&self, square: BoardSquare) -> Bitboard {
        self.attack_tables[square.enumeration()]
    }

    fn generate_attack_tables(piece: Piece, side: Side) -> [Bitboard; 64] {
        let mut attack_tables: [Bitboard; 64] = [Bitboard::new(0); 64];

        match piece {
            Piece::Pawn | Piece::Knight | Piece::King => {
                BoardSquare::iter().for_each(|square| {
                    attack_tables[square.enumeration()] =
                        Self::generate_leaper_attack_table(piece, side, square);
                });
            }
            _ => {
                BoardSquare::iter().for_each(|square| {
                    attack_tables[square.enumeration()] =
                        Self::generate_slider_attack_table(piece, square);
                });
            }
        }

        attack_tables
    }

    fn generate_leaper_attack_table(piece: Piece, side: Side, square: BoardSquare) -> Bitboard {
        // Bitboards with all values initialised to 1, except for the file(s) indicated
        // Used to prevent incorrect attack table generation for pieces on / near edge files
        let file_a_zeroed = Bitboard::new(18374403900871474942);
        let file_h_zeroed = Bitboard::new(9187201950435737471);
        let file_ab_zeroed = Bitboard::new(18229723555195321596);
        let file_gh_zeroed = Bitboard::new(4557430888798830399);

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
            _ => {}
        }

        attack_table
    }

    fn generate_slider_attack_table(piece: Piece, square: BoardSquare) -> Bitboard {
        let mut attack_table = Bitboard::new(0);

        let target_rank = (square.enumeration()) / 8;
        let target_file = (square.enumeration()) % 8;

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

        attack_table
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attack_tables_white_pawn() {
        let attack_tables = AttackTables::new(Piece::Pawn, Side::White);

        let desired_h3_attack_table = u64::pow(2, 38);
        let desired_f5_attack_table = u64::pow(2, 20) + u64::pow(2, 22);
        let desired_a4_attack_table = u64::pow(2, 25);

        assert_eq!(
            attack_tables.attack_table(BoardSquare::H3).bitboard,
            desired_h3_attack_table
        );
        assert_eq!(
            attack_tables.attack_table(BoardSquare::F5).bitboard,
            desired_f5_attack_table
        );
        assert_eq!(
            attack_tables.attack_table(BoardSquare::A4).bitboard,
            desired_a4_attack_table
        );
    }

    #[test]
    fn attack_tables_black_pawn() {
        let attack_tables = AttackTables::new(Piece::Pawn, Side::Black);

        let desired_b4_attack_table = u64::pow(2, 40) + u64::pow(2, 42);
        let desired_h4_attack_table = u64::pow(2, 46);
        let desired_a5_attack_table = u64::pow(2, 33);

        assert_eq!(
            attack_tables.attack_table(BoardSquare::B4).bitboard,
            desired_b4_attack_table
        );
        assert_eq!(
            attack_tables.attack_table(BoardSquare::H4).bitboard,
            desired_h4_attack_table
        );
        assert_eq!(
            attack_tables.attack_table(BoardSquare::A5).bitboard,
            desired_a5_attack_table
        );
    }

    #[test]
    fn attack_tables_knight() {
        let attack_tables = AttackTables::new(Piece::Knight, Side::Either);

        let desired_g5_attack_table = u64::pow(2, 13)
            + u64::pow(2, 15)
            + u64::pow(2, 20)
            + u64::pow(2, 36)
            + u64::pow(2, 45)
            + u64::pow(2, 47);
        let desired_e2_attack_table = u64::pow(2, 35)
            + u64::pow(2, 37)
            + u64::pow(2, 42)
            + u64::pow(2, 46)
            + u64::pow(2, 58)
            + u64::pow(2, 62);
        let desired_f4_attack_table = u64::pow(2, 20)
            + u64::pow(2, 22)
            + u64::pow(2, 27)
            + u64::pow(2, 31)
            + u64::pow(2, 43)
            + u64::pow(2, 47)
            + u64::pow(2, 52)
            + u64::pow(2, 54);
        let desired_b4_attack_table = u64::pow(2, 16)
            + u64::pow(2, 18)
            + u64::pow(2, 27)
            + u64::pow(2, 43)
            + u64::pow(2, 48)
            + u64::pow(2, 50);
        let desired_a4_attack_table =
            u64::pow(2, 17) + u64::pow(2, 26) + u64::pow(2, 42) + u64::pow(2, 49);
        let desired_h8_attack_table = u64::pow(2, 13) + u64::pow(2, 22);

        assert_eq!(
            attack_tables.attack_table(BoardSquare::G5).bitboard,
            desired_g5_attack_table
        );
        assert_eq!(
            attack_tables.attack_table(BoardSquare::E2).bitboard,
            desired_e2_attack_table
        );
        assert_eq!(
            attack_tables.attack_table(BoardSquare::F4).bitboard,
            desired_f4_attack_table
        );
        assert_eq!(
            attack_tables.attack_table(BoardSquare::B4).bitboard,
            desired_b4_attack_table
        );
        assert_eq!(
            attack_tables.attack_table(BoardSquare::A4).bitboard,
            desired_a4_attack_table
        );
        assert_eq!(
            attack_tables.attack_table(BoardSquare::H8).bitboard,
            desired_h8_attack_table
        );
    }
}
