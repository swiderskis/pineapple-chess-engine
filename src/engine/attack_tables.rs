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
            Piece::Bishop | Piece::Rook | Piece::Queen => {
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

        let piece_rank = square.rank();
        let piece_file = square.file();

        // Cardinal occupancy
        if matches!(piece, Piece::Rook) || matches!(piece, Piece::Queen) {
            for rank in (piece_rank + 1)..7 {
                attack_table.bitboard |= 1 << rank * 8 + piece_file;
            }

            for rank in (1..piece_rank).rev() {
                attack_table.bitboard |= 1 << rank * 8 + piece_file;
            }

            for file in (piece_file + 1)..7 {
                attack_table.bitboard |= 1 << piece_rank * 8 + file;
            }

            for file in (1..piece_file).rev() {
                attack_table.bitboard |= 1 << piece_rank * 8 + file;
            }
        }

        // Diagonal occupancy
        if matches!(piece, Piece::Bishop) || matches!(piece, Piece::Queen) {
            for (rank, file) in ((piece_rank + 1)..7).zip((piece_file + 1)..7) {
                attack_table.bitboard |= 1 << (rank * 8 + file);
            }

            for (rank, file) in ((1..piece_rank).rev()).zip((piece_file + 1)..7) {
                attack_table.bitboard |= 1 << (rank * 8 + file);
            }

            for (rank, file) in ((piece_rank + 1)..7).zip((1..piece_file).rev()) {
                attack_table.bitboard |= 1 << (rank * 8 + file);
            }

            for (rank, file) in ((1..piece_rank).rev()).zip((1..piece_file).rev()) {
                attack_table.bitboard |= 1 << (rank * 8 + file);
            }
        }

        attack_table
    }

    fn generate_current_slider_attack_table(
        piece: Piece,
        square: BoardSquare,
        board: Bitboard,
    ) -> Bitboard {
        let mut attack_table = Bitboard::new(0);

        let piece_rank = square.rank();
        let piece_file = square.file();

        // Cardinal occupancy
        if matches!(piece, Piece::Rook) || matches!(piece, Piece::Queen) {
            for rank in (piece_rank + 1)..8 {
                attack_table.bitboard |= 1 << rank * 8 + piece_file;

                if (1 << rank * 8 + piece_file) & board.bitboard != 0 {
                    break;
                }
            }

            for rank in (0..piece_rank).rev() {
                attack_table.bitboard |= 1 << rank * 8 + piece_file;

                if (1 << rank * 8 + piece_file) & board.bitboard != 0 {
                    break;
                }
            }

            for file in (piece_file + 1)..8 {
                attack_table.bitboard |= 1 << piece_rank * 8 + file;

                if (1 << piece_rank * 8 + file) & board.bitboard != 0 {
                    break;
                }
            }

            for file in (0..piece_file).rev() {
                attack_table.bitboard |= 1 << piece_rank * 8 + file;

                if (1 << piece_rank * 8 + file) & board.bitboard != 0 {
                    break;
                }
            }
        }

        // Diagonal occupancy
        if matches!(piece, Piece::Bishop) || matches!(piece, Piece::Queen) {
            for (rank, file) in ((piece_rank + 1)..8).zip((piece_file + 1)..8) {
                attack_table.bitboard |= 1 << (rank * 8 + file);

                if (1 << (rank * 8 + file)) & board.bitboard != 0 {
                    break;
                }
            }

            for (rank, file) in ((0..piece_rank).rev()).zip((piece_file + 1)..8) {
                attack_table.bitboard |= 1 << (rank * 8 + file);

                if (1 << (rank * 8 + file)) & board.bitboard != 0 {
                    break;
                }
            }

            for (rank, file) in ((piece_rank + 1)..8).zip((0..piece_file).rev()) {
                attack_table.bitboard |= 1 << (rank * 8 + file);

                if (1 << (rank * 8 + file)) & board.bitboard != 0 {
                    break;
                }
            }

            for (rank, file) in ((0..piece_rank).rev()).zip((0..piece_file).rev()) {
                attack_table.bitboard |= 1 << (rank * 8 + file);

                if (1 << (rank * 8 + file)) & board.bitboard != 0 {
                    break;
                }
            }
        }

        attack_table
    }

    // Implementation to generate magic numbers taken from
    // https://www.youtube.com/watch?v=UnEu5GOiSEs&list=PLmN0neTso3Jxh8ZIylk74JpwfiWNI76Cs&index=15
    // NB this seems to take much longer for me - no clue why, must be some problem in the code I can't see
    // Not too important as magic numbers will be hard coded anyway
    pub fn generate_magic_number(
        mut random_state: &mut u32,
        attack_table: Bitboard,
        piece: Piece,
        square: BoardSquare,
    ) -> u64 {
        let occupancy_count = match piece {
            Piece::Bishop => [
                6, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 7, 9,
                9, 7, 5, 5, 5, 5, 7, 9, 9, 7, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
                6, 5, 5, 5, 5, 5, 5, 6,
            ],
            Piece::Rook => [
                12, 11, 11, 11, 11, 11, 11, 12, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10,
                10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10,
                10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 12, 11, 11, 11, 11, 11, 11,
                12,
            ],
            _ => panic!("Attempted to generate magic number for non-rook / bishop piece"),
        };

        let mut occupancies: [Bitboard; 4096] = [Bitboard::new(0); 4096];
        let mut attacks: [Bitboard; 4096] = [Bitboard::new(0); 4096];

        let occupancy_indices = 1 << occupancy_count[square.enumeration()];

        for i in 0..occupancy_indices {
            occupancies[i] = Self::set_occupancy(i, attack_table);
            attacks[i] =
                AttackTables::generate_current_slider_attack_table(piece, square, occupancies[i]);
        }

        'outer: loop {
            let magic_number_candidate = Self::generate_magic_number_candidate(&mut random_state);

            if (attack_table
                .bitboard
                .overflowing_mul(magic_number_candidate)
                .0
                & 0xFF00000000000000)
                .count_ones()
                < 6
            {
                continue;
            };

            let mut used_attacks: [Bitboard; 4096] = [Bitboard::new(0); 4096];

            for i in 0..occupancy_indices {
                let magic_index = ((occupancies[i]
                    .bitboard
                    .overflowing_mul(magic_number_candidate)
                    .0)
                    >> (64 - occupancy_count[square.enumeration()]))
                    as usize;

                if used_attacks[magic_index].bitboard == 0 {
                    used_attacks[magic_index].bitboard = attacks[i].bitboard;
                } else if used_attacks[magic_index].bitboard != attacks[i].bitboard {
                    continue 'outer;
                }
            }

            return magic_number_candidate;
        }
    }

    fn generate_magic_number_candidate(mut random_seed: &mut u32) -> u64 {
        random_numbers::generate_random_u64_integer(&mut random_seed)
            & random_numbers::generate_random_u64_integer(&mut random_seed)
            & random_numbers::generate_random_u64_integer(&mut random_seed)
    }

    fn set_occupancy(index: usize, attack_table: Bitboard) -> Bitboard {
        let mut occupancy = Bitboard::new(0);

        let mut attack_table_clone = attack_table.clone();
        let mut count = 0;

        loop {
            let ls1b_square = match attack_table_clone.get_ls1b_index() {
                Some(index) => BoardSquare::new_from_index(index),
                None => break,
            };

            if index & (1 << count) != 0 {
                occupancy.bitboard |= 1 << ls1b_square.enumeration();
            }

            attack_table_clone.pop_bit(ls1b_square);
            count += 1;
        }

        occupancy
    }
}

pub mod random_numbers {
    pub fn generate_random_u64_integer(mut random_seed: &mut u32) -> u64 {
        // `& 0xFFFF` operation cuts off first 16 most significant bits from 32 bit integer
        mutate_random_state(&mut random_seed);
        let random_u64_integer_1 = (*random_seed & 0xFFFF) as u64;

        mutate_random_state(&mut random_seed);
        let random_u64_integer_2 = (*random_seed & 0xFFFF) as u64;

        mutate_random_state(&mut random_seed);
        let random_u64_integer_3 = (*random_seed & 0xFFFF) as u64;

        mutate_random_state(&mut random_seed);
        let random_u64_integer_4 = (*random_seed & 0xFFFF) as u64;

        random_u64_integer_1
            | (random_u64_integer_2 << 16)
            | (random_u64_integer_3 << 32)
            | (random_u64_integer_4 << 48)
    }

    fn mutate_random_state(random_state: &mut u32) {
        // XOR shift algorithm
        *random_state ^= *random_state << 13;
        *random_state ^= *random_state >> 17;
        *random_state ^= *random_state << 5;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attack_tables_white_pawn() {
        let attack_tables = AttackTables::new(Piece::Pawn, Side::White);

        let desired_h3_attack_table = u64::pow(2, BoardSquare::G4 as u32);
        let desired_f5_attack_table =
            u64::pow(2, BoardSquare::E6 as u32) + u64::pow(2, BoardSquare::G6 as u32);
        let desired_a4_attack_table = u64::pow(2, BoardSquare::B5 as u32);

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

        let desired_b4_attack_table =
            u64::pow(2, BoardSquare::A3 as u32) + u64::pow(2, BoardSquare::C3 as u32);
        let desired_h4_attack_table = u64::pow(2, BoardSquare::G3 as u32);
        let desired_a5_attack_table = u64::pow(2, BoardSquare::B4 as u32);

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

        let desired_g5_attack_table = u64::pow(2, BoardSquare::F7 as u32)
            + u64::pow(2, BoardSquare::H7 as u32)
            + u64::pow(2, BoardSquare::E6 as u32)
            + u64::pow(2, BoardSquare::E4 as u32)
            + u64::pow(2, BoardSquare::F3 as u32)
            + u64::pow(2, BoardSquare::H3 as u32);
        let desired_e2_attack_table = u64::pow(2, BoardSquare::D4 as u32)
            + u64::pow(2, BoardSquare::F4 as u32)
            + u64::pow(2, BoardSquare::C3 as u32)
            + u64::pow(2, BoardSquare::G3 as u32)
            + u64::pow(2, BoardSquare::C1 as u32)
            + u64::pow(2, BoardSquare::G1 as u32);
        let desired_f4_attack_table = u64::pow(2, BoardSquare::E6 as u32)
            + u64::pow(2, BoardSquare::G6 as u32)
            + u64::pow(2, BoardSquare::D5 as u32)
            + u64::pow(2, BoardSquare::H5 as u32)
            + u64::pow(2, BoardSquare::D3 as u32)
            + u64::pow(2, BoardSquare::H3 as u32)
            + u64::pow(2, BoardSquare::E2 as u32)
            + u64::pow(2, BoardSquare::G2 as u32);
        let desired_b4_attack_table = u64::pow(2, BoardSquare::A6 as u32)
            + u64::pow(2, BoardSquare::C6 as u32)
            + u64::pow(2, BoardSquare::D5 as u32)
            + u64::pow(2, BoardSquare::D3 as u32)
            + u64::pow(2, BoardSquare::A2 as u32)
            + u64::pow(2, BoardSquare::C2 as u32);
        let desired_a4_attack_table = u64::pow(2, BoardSquare::B6 as u32)
            + u64::pow(2, BoardSquare::C5 as u32)
            + u64::pow(2, BoardSquare::C3 as u32)
            + u64::pow(2, BoardSquare::B2 as u32);
        let desired_h8_attack_table =
            u64::pow(2, BoardSquare::F7 as u32) + u64::pow(2, BoardSquare::G6 as u32);

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

    #[test]
    fn attack_tables_king() {
        let attack_tables = AttackTables::new(Piece::King, Side::Either);

        let desired_b2_attack_table = u64::pow(2, BoardSquare::A3 as u32)
            + u64::pow(2, BoardSquare::B3 as u32)
            + u64::pow(2, BoardSquare::C3 as u32)
            + u64::pow(2, BoardSquare::A2 as u32)
            + u64::pow(2, BoardSquare::C2 as u32)
            + u64::pow(2, BoardSquare::A1 as u32)
            + u64::pow(2, BoardSquare::B1 as u32)
            + u64::pow(2, BoardSquare::C1 as u32);
        let desired_a1_attack_table = u64::pow(2, BoardSquare::A2 as u32)
            + u64::pow(2, BoardSquare::B2 as u32)
            + u64::pow(2, BoardSquare::B1 as u32);
        let desired_h4_attack_table = u64::pow(2, BoardSquare::G5 as u32)
            + u64::pow(2, BoardSquare::H5 as u32)
            + u64::pow(2, BoardSquare::G4 as u32)
            + u64::pow(2, BoardSquare::G3 as u32)
            + u64::pow(2, BoardSquare::H3 as u32);

        assert_eq!(
            attack_tables.attack_table(BoardSquare::B2).bitboard,
            desired_b2_attack_table
        );
        assert_eq!(
            attack_tables.attack_table(BoardSquare::A1).bitboard,
            desired_a1_attack_table
        );
        assert_eq!(
            attack_tables.attack_table(BoardSquare::H4).bitboard,
            desired_h4_attack_table
        );
    }
}
