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

    pub fn attack_tables(self) -> [Bitboard; 64] {
        self.attack_tables
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

            attack_tables[square.enumeration()].bitboard = attack_table.bitboard;
        });

        attack_tables
    }

    fn generate_slider_attack_table(piece: Piece, square: BoardSquare) -> u64 {
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

        attack_table.bitboard
    }
}
