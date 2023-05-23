use super::{Bitboard, BoardSquare, Piece, Side};
use strum::IntoEnumIterator;

pub struct LeaperAttackTables {
    white_pawn_attack_tables: [Bitboard; 64],
    black_pawn_attack_tables: [Bitboard; 64],
    knight_attack_tables: [Bitboard; 64],
    king_attack_tables: [Bitboard; 64],
}

pub struct SliderAttackTables {
    bishop_attack_masks: [Bitboard; 64],
    rook_attack_masks: [Bitboard; 64],
    // bishop_attack_tables: [[Bitboard; 64]; 512],
    // rook_attack_tables: [[Bitboard; 64]; 4096],
}

pub trait AttackTables {
    fn initialise() -> Self;

    fn attack_mask(&self, piece: &Piece, side: &Side, square: &BoardSquare) -> Bitboard;

    fn generate_attack_masks(piece: Piece, side: Side) -> [Bitboard; 64];
}

impl AttackTables for LeaperAttackTables {
    fn initialise() -> Self {
        Self {
            white_pawn_attack_tables: Self::generate_attack_masks(Piece::Pawn, Side::White),
            black_pawn_attack_tables: Self::generate_attack_masks(Piece::Pawn, Side::Black),
            knight_attack_tables: Self::generate_attack_masks(Piece::Knight, Side::Either),
            king_attack_tables: Self::generate_attack_masks(Piece::King, Side::Either),
        }
    }

    fn attack_mask(&self, piece: &Piece, side: &Side, square: &BoardSquare) -> Bitboard {
        match piece {
            Piece::Pawn => match side {
                Side::White => self.white_pawn_attack_tables[square.enumeration()],
                Side::Black => self.black_pawn_attack_tables[square.enumeration()],
                Side::Either => {
                    panic!("Attempted to access pawn attack table with side == Side::Either")
                }
            },
            Piece::Knight => self.knight_attack_tables[square.enumeration()],
            Piece::King => self.king_attack_tables[square.enumeration()],
            _ => {
                panic!("Attempted to access slider attack table on leaper attack tables struct")
            }
        }
    }

    fn generate_attack_masks(piece: Piece, side: Side) -> [Bitboard; 64] {
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

            bitboard.set_bit(&square);

            attack_tables[square.enumeration()] = match piece {
                Piece::Pawn => {
                    if matches!(side, Side::White) {
                        attack_table.bitboard |= (bitboard.bitboard >> 7) & file_a_zeroed.bitboard;
                        attack_table.bitboard |= (bitboard.bitboard >> 9) & file_h_zeroed.bitboard;
                    } else {
                        attack_table.bitboard |= (bitboard.bitboard << 7) & file_h_zeroed.bitboard;
                        attack_table.bitboard |= (bitboard.bitboard << 9) & file_a_zeroed.bitboard;
                    }

                    attack_table
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

                    attack_table
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

                    attack_table
                }
                _ => panic!("Attempted to initialise leaper piece attack mask for slider piece"),
            }
        });

        attack_tables
    }
}

impl AttackTables for SliderAttackTables {
    fn initialise() -> Self {
        let bishop_attack_masks = Self::generate_attack_masks(Piece::Bishop, Side::Either);
        let rook_attack_masks = Self::generate_attack_masks(Piece::Rook, Side::Either);

        Self {
            bishop_attack_masks,
            rook_attack_masks,
        }
    }

    fn attack_mask(&self, piece: &Piece, _side: &Side, square: &BoardSquare) -> Bitboard {
        match piece {
            Piece::Bishop => self.bishop_attack_masks[square.enumeration()],
            Piece::Rook => self.rook_attack_masks[square.enumeration()],
            _ => {
                panic!("Attempted to access leaper attack table on slider attack tables struct")
            }
        }
    }

    fn generate_attack_masks(piece: Piece, _side: Side) -> [Bitboard; 64] {
        let mut attack_tables: [Bitboard; 64] = [Bitboard::new(0); 64];

        BoardSquare::iter().for_each(|square| {
            let mut attack_table = Bitboard::new(0);

            let piece_rank = square.rank();
            let piece_file = square.file();

            attack_tables[square.enumeration()] = match piece {
                Piece::Bishop => {
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

                    attack_table
                }
                Piece::Rook => {
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

                    attack_table
                }
                _ => panic!("Attempted to initialise slide piece attack mask for leaper piece"),
            }
        });

        attack_tables
    }
}

impl SliderAttackTables {
    fn generate_current_slider_attack_table(
        piece: &Piece,
        square: &BoardSquare,
        board: &Bitboard,
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
}

pub mod magic_numbers {
    use super::{Bitboard, BoardSquare, Piece, SliderAttackTables};

    // Implementation to generate magic numbers taken from
    // https://www.youtube.com/watch?v=UnEu5GOiSEs&list=PLmN0neTso3Jxh8ZIylk74JpwfiWNI76Cs&index=15
    // NB this seems to take much longer for me - no clue why, must be some problem in the code I can't see
    // Not too important as magic numbers are hard coded anyway
    pub fn _generate_magic_number(
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
            occupancies[i] = _set_occupancy(i, attack_table);
            attacks[i] = SliderAttackTables::generate_current_slider_attack_table(
                &piece,
                &square,
                &occupancies[i],
            );
        }

        'outer: loop {
            let magic_number_candidate = _generate_random_u64_integer(&mut random_state)
                & _generate_random_u64_integer(&mut random_state)
                & _generate_random_u64_integer(&mut random_state);

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

    fn _set_occupancy(index: usize, attack_table: Bitboard) -> Bitboard {
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

            attack_table_clone.pop_bit(&ls1b_square);
            count += 1;
        }

        occupancy
    }

    fn _generate_random_u64_integer(mut random_state: &mut u32) -> u64 {
        // `& 0xFFFF` operation cuts off first 16 most significant bits from 32 bit integer
        _mutate_random_state(&mut random_state);
        let random_u64_integer_1 = (*random_state & 0xFFFF) as u64;

        _mutate_random_state(&mut random_state);
        let random_u64_integer_2 = (*random_state & 0xFFFF) as u64;

        _mutate_random_state(&mut random_state);
        let random_u64_integer_3 = (*random_state & 0xFFFF) as u64;

        _mutate_random_state(&mut random_state);
        let random_u64_integer_4 = (*random_state & 0xFFFF) as u64;

        random_u64_integer_1
            | (random_u64_integer_2 << 16)
            | (random_u64_integer_3 << 32)
            | (random_u64_integer_4 << 48)
    }

    fn _mutate_random_state(random_state: &mut u32) {
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
        let attack_tables = LeaperAttackTables::initialise();

        let desired_h3_attack_table = u64::pow(2, BoardSquare::G4 as u32);
        let desired_f5_attack_table =
            u64::pow(2, BoardSquare::E6 as u32) + u64::pow(2, BoardSquare::G6 as u32);
        let desired_a4_attack_table = u64::pow(2, BoardSquare::B5 as u32);

        assert_eq!(
            attack_tables
                .attack_mask(&Piece::Pawn, &Side::White, &BoardSquare::H3)
                .bitboard,
            desired_h3_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_mask(&Piece::Pawn, &Side::White, &BoardSquare::F5)
                .bitboard,
            desired_f5_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_mask(&Piece::Pawn, &Side::White, &BoardSquare::A4)
                .bitboard,
            desired_a4_attack_table
        );
    }

    #[test]
    fn attack_tables_black_pawn() {
        let attack_tables = LeaperAttackTables::initialise();

        let desired_b4_attack_table =
            u64::pow(2, BoardSquare::A3 as u32) + u64::pow(2, BoardSquare::C3 as u32);
        let desired_h4_attack_table = u64::pow(2, BoardSquare::G3 as u32);
        let desired_a5_attack_table = u64::pow(2, BoardSquare::B4 as u32);

        assert_eq!(
            attack_tables
                .attack_mask(&Piece::Pawn, &Side::Black, &BoardSquare::B4)
                .bitboard,
            desired_b4_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_mask(&Piece::Pawn, &Side::Black, &BoardSquare::H4)
                .bitboard,
            desired_h4_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_mask(&Piece::Pawn, &Side::Black, &BoardSquare::A5)
                .bitboard,
            desired_a5_attack_table
        );
    }

    #[test]
    fn attack_tables_knight() {
        let attack_tables = LeaperAttackTables::initialise();

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
            attack_tables
                .attack_mask(&Piece::Knight, &Side::Either, &BoardSquare::G5)
                .bitboard,
            desired_g5_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_mask(&Piece::Knight, &Side::Either, &BoardSquare::E2)
                .bitboard,
            desired_e2_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_mask(&Piece::Knight, &Side::Either, &BoardSquare::F4)
                .bitboard,
            desired_f4_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_mask(&Piece::Knight, &Side::Either, &BoardSquare::B4)
                .bitboard,
            desired_b4_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_mask(&Piece::Knight, &Side::Either, &BoardSquare::A4)
                .bitboard,
            desired_a4_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_mask(&Piece::Knight, &Side::Either, &BoardSquare::H8)
                .bitboard,
            desired_h8_attack_table
        );
    }

    #[test]
    fn attack_tables_bishop() {
        let attack_tables = SliderAttackTables::initialise();

        let desired_a5_attack_table = u64::pow(2, BoardSquare::B6 as u32)
            + u64::pow(2, BoardSquare::C7 as u32)
            + u64::pow(2, BoardSquare::B4 as u32)
            + u64::pow(2, BoardSquare::C3 as u32)
            + u64::pow(2, BoardSquare::D2 as u32);
        let desired_g7_attack_table = u64::pow(2, BoardSquare::F6 as u32)
            + u64::pow(2, BoardSquare::E5 as u32)
            + u64::pow(2, BoardSquare::D4 as u32)
            + u64::pow(2, BoardSquare::C3 as u32)
            + u64::pow(2, BoardSquare::B2 as u32);
        let desired_d6_attack_table = u64::pow(2, BoardSquare::C7 as u32)
            + u64::pow(2, BoardSquare::E7 as u32)
            + u64::pow(2, BoardSquare::C5 as u32)
            + u64::pow(2, BoardSquare::B4 as u32)
            + u64::pow(2, BoardSquare::E5 as u32)
            + u64::pow(2, BoardSquare::F4 as u32)
            + u64::pow(2, BoardSquare::G3 as u32);

        assert_eq!(
            attack_tables
                .attack_mask(&Piece::Bishop, &Side::Either, &BoardSquare::A5)
                .bitboard,
            desired_a5_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_mask(&Piece::Bishop, &Side::Either, &BoardSquare::G7)
                .bitboard,
            desired_g7_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_mask(&Piece::Bishop, &Side::Either, &BoardSquare::D6)
                .bitboard,
            desired_d6_attack_table
        );
    }

    #[test]
    fn attack_tables_rook() {
        let attack_tables = SliderAttackTables::initialise();

        let desired_d5_attack_table = u64::pow(2, BoardSquare::D7 as u32)
            + u64::pow(2, BoardSquare::D6 as u32)
            + u64::pow(2, BoardSquare::B5 as u32)
            + u64::pow(2, BoardSquare::C5 as u32)
            + u64::pow(2, BoardSquare::E5 as u32)
            + u64::pow(2, BoardSquare::F5 as u32)
            + u64::pow(2, BoardSquare::G5 as u32)
            + u64::pow(2, BoardSquare::D4 as u32)
            + u64::pow(2, BoardSquare::D3 as u32)
            + u64::pow(2, BoardSquare::D2 as u32);
        let desired_b3_attack_table = u64::pow(2, BoardSquare::B7 as u32)
            + u64::pow(2, BoardSquare::B6 as u32)
            + u64::pow(2, BoardSquare::B5 as u32)
            + u64::pow(2, BoardSquare::B4 as u32)
            + u64::pow(2, BoardSquare::B2 as u32)
            + u64::pow(2, BoardSquare::C3 as u32)
            + u64::pow(2, BoardSquare::D3 as u32)
            + u64::pow(2, BoardSquare::E3 as u32)
            + u64::pow(2, BoardSquare::F3 as u32)
            + u64::pow(2, BoardSquare::G3 as u32);
        let desired_e1_attack_table = u64::pow(2, BoardSquare::E7 as u32)
            + u64::pow(2, BoardSquare::E6 as u32)
            + u64::pow(2, BoardSquare::E5 as u32)
            + u64::pow(2, BoardSquare::E4 as u32)
            + u64::pow(2, BoardSquare::E3 as u32)
            + u64::pow(2, BoardSquare::E2 as u32)
            + u64::pow(2, BoardSquare::B1 as u32)
            + u64::pow(2, BoardSquare::C1 as u32)
            + u64::pow(2, BoardSquare::D1 as u32)
            + u64::pow(2, BoardSquare::F1 as u32)
            + u64::pow(2, BoardSquare::G1 as u32);

        assert_eq!(
            attack_tables
                .attack_mask(&Piece::Rook, &Side::Either, &BoardSquare::D5)
                .bitboard,
            desired_d5_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_mask(&Piece::Rook, &Side::Either, &BoardSquare::B3)
                .bitboard,
            desired_b3_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_mask(&Piece::Rook, &Side::Either, &BoardSquare::E1)
                .bitboard,
            desired_e1_attack_table
        );
    }

    #[test]
    fn attack_tables_king() {
        let attack_tables = LeaperAttackTables::initialise();

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
            attack_tables
                .attack_mask(&Piece::King, &Side::Either, &BoardSquare::B2)
                .bitboard,
            desired_b2_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_mask(&Piece::King, &Side::Either, &BoardSquare::A1)
                .bitboard,
            desired_a1_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_mask(&Piece::King, &Side::Either, &BoardSquare::H4)
                .bitboard,
            desired_h4_attack_table
        );
    }
}
