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
    bishop_attack_tables: Vec<[Bitboard; 512]>,
    rook_attack_tables: Vec<[Bitboard; 4096]>,
}

pub trait AttackTablesPub {
    fn initialise() -> Self;

    fn attack_table(
        &self,
        board: &Bitboard,
        piece: &Piece,
        side: &Side,
        square: &BoardSquare,
    ) -> Bitboard;
}

trait AttackTables {
    fn attack_mask(&self, piece: &Piece, side: &Side, square: &BoardSquare) -> Bitboard;

    fn generate_attack_masks(piece: Piece, side: Side) -> [Bitboard; 64];
}

impl AttackTablesPub for LeaperAttackTables {
    fn initialise() -> Self {
        Self {
            white_pawn_attack_tables: Self::generate_attack_masks(Piece::Pawn, Side::White),
            black_pawn_attack_tables: Self::generate_attack_masks(Piece::Pawn, Side::Black),
            knight_attack_tables: Self::generate_attack_masks(Piece::Knight, Side::Either),
            king_attack_tables: Self::generate_attack_masks(Piece::King, Side::Either),
        }
    }

    fn attack_table(
        &self,
        _board: &Bitboard,
        piece: &Piece,
        side: &Side,
        square: &BoardSquare,
    ) -> Bitboard {
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
                panic!("Attempted to access slider attack table on leaper attack tables")
            }
        }
    }
}

impl AttackTables for LeaperAttackTables {
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
                panic!("Attempted to access slider attack mask on leaper attack masks")
            }
        }
    }
}

impl AttackTablesPub for SliderAttackTables {
    fn initialise() -> Self {
        let bishop_attack_masks = Self::generate_attack_masks(Piece::Bishop, Side::Either);
        let rook_attack_masks = Self::generate_attack_masks(Piece::Rook, Side::Either);

        let mut bishop_attack_tables = vec![[Bitboard::new(0); 512]; 64];
        let mut rook_attack_tables = vec![[Bitboard::new(0); 4096]; 64];

        let magic_numbers = MagicNumbers::initialise();

        BoardSquare::iter().for_each(|square| {
            let bishop_occupancy_indices =
                1 << bishop_attack_masks[square.enumeration()].count_bits();

            for i in 0..bishop_occupancy_indices {
                let occupancy = Self::set_occupancy(i, bishop_attack_masks[square.enumeration()]);

                let magic_index = magic_numbers.generate_magic_index(
                    &occupancy,
                    &bishop_attack_masks[square.enumeration()],
                    &square,
                    &Piece::Bishop,
                );

                bishop_attack_tables[square.enumeration()][magic_index] =
                    Self::generate_attack_table(&occupancy, &Piece::Bishop, &square);
            }

            let rook_occupancy_indices = 1 << rook_attack_masks[square.enumeration()].count_bits();

            for i in 0..rook_occupancy_indices {
                let occupancy = Self::set_occupancy(i, rook_attack_masks[square.enumeration()]);

                let magic_index = magic_numbers.generate_magic_index(
                    &occupancy,
                    &rook_attack_masks[square.enumeration()],
                    &square,
                    &Piece::Rook,
                );

                rook_attack_tables[square.enumeration()][magic_index] =
                    Self::generate_attack_table(&occupancy, &Piece::Rook, &square);
            }
        });

        Self {
            bishop_attack_masks,
            rook_attack_masks,
            bishop_attack_tables,
            rook_attack_tables,
        }
    }

    fn attack_table(
        &self,
        board: &Bitboard,
        piece: &Piece,
        side: &Side,
        square: &BoardSquare,
    ) -> Bitboard {
        let mut board_clone = board.clone();
        let magic_numbers = MagicNumbers::initialise();

        board_clone.bitboard &= self.attack_mask(&piece, &side, &square).bitboard;
        board_clone.bitboard = board_clone
            .bitboard
            .overflowing_mul(magic_numbers.magic_number(&piece, &square))
            .0;
        board_clone.bitboard >>= 64 - self.attack_mask(&piece, &side, &square).count_bits();

        match piece {
            Piece::Bishop => {
                self.bishop_attack_tables[square.enumeration()][board_clone.bitboard as usize]
            }
            Piece::Rook => {
                self.rook_attack_tables[square.enumeration()][board_clone.bitboard as usize]
            }
            _ => panic!("Attempted to access attack table for non-slider piece"),
        }
    }
}

impl AttackTables for SliderAttackTables {
    fn attack_mask(&self, piece: &Piece, _side: &Side, square: &BoardSquare) -> Bitboard {
        match piece {
            Piece::Bishop => self.bishop_attack_masks[square.enumeration()],
            Piece::Rook => self.rook_attack_masks[square.enumeration()],
            _ => {
                panic!("Attempted to access leaper attack mask on slider attack masks")
            }
        }
    }

    fn generate_attack_masks(piece: Piece, _side: Side) -> [Bitboard; 64] {
        let mut attack_masks: [Bitboard; 64] = [Bitboard::new(0); 64];

        BoardSquare::iter().for_each(|square| {
            let mut attack_mask = Bitboard::new(0);

            let piece_rank = square.rank();
            let piece_file = square.file();

            match piece {
                Piece::Bishop => {
                    for (rank, file) in ((piece_rank + 1)..7).zip((piece_file + 1)..7) {
                        attack_mask.bitboard |= 1 << (rank * 8 + file);
                    }

                    for (rank, file) in ((1..piece_rank).rev()).zip((piece_file + 1)..7) {
                        attack_mask.bitboard |= 1 << (rank * 8 + file);
                    }

                    for (rank, file) in ((piece_rank + 1)..7).zip((1..piece_file).rev()) {
                        attack_mask.bitboard |= 1 << (rank * 8 + file);
                    }

                    for (rank, file) in ((1..piece_rank).rev()).zip((1..piece_file).rev()) {
                        attack_mask.bitboard |= 1 << (rank * 8 + file);
                    }

                    attack_masks[square.enumeration()] = attack_mask;
                }
                Piece::Rook => {
                    for rank in (piece_rank + 1)..7 {
                        attack_mask.bitboard |= 1 << rank * 8 + piece_file;
                    }

                    for rank in (1..piece_rank).rev() {
                        attack_mask.bitboard |= 1 << rank * 8 + piece_file;
                    }

                    for file in (piece_file + 1)..7 {
                        attack_mask.bitboard |= 1 << piece_rank * 8 + file;
                    }

                    for file in (1..piece_file).rev() {
                        attack_mask.bitboard |= 1 << piece_rank * 8 + file;
                    }

                    attack_masks[square.enumeration()] = attack_mask;
                }
                _ => panic!("Attempted to initialise slide piece attack mask for leaper piece"),
            }
        });

        attack_masks
    }
}

impl SliderAttackTables {
    fn generate_attack_table(board: &Bitboard, piece: &Piece, square: &BoardSquare) -> Bitboard {
        let mut attack_table = Bitboard::new(0);

        let piece_rank = square.rank();
        let piece_file = square.file();

        match piece {
            Piece::Bishop => {
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
            Piece::Rook => {
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
            _ => panic!("Attempted to generate attack table for non-slider piece"),
        }

        attack_table
    }

    fn set_occupancy(index: usize, attack_mask: Bitboard) -> Bitboard {
        let mut occupancy = Bitboard::new(0);

        let mut attack_mask_clone = attack_mask.clone();
        let mut count = 0;

        loop {
            let ls1b_square = match attack_mask_clone.get_ls1b_index() {
                Some(index) => BoardSquare::new_from_index(index),
                None => break,
            };

            if index & (1 << count) != 0 {
                occupancy.bitboard |= 1 << ls1b_square.enumeration();
            }

            attack_mask_clone.pop_bit(&ls1b_square);
            count += 1;
        }

        occupancy
    }
}

struct MagicNumbers {
    bishop_magic_numbers: [u64; 64],
    rook_magic_numbers: [u64; 64],
}

impl MagicNumbers {
    pub fn initialise() -> Self {
        // Magic numbers generated using attack_tables::generate_magic_numbers module,
        // with random_state = 1804289383
        Self {
            bishop_magic_numbers: [
                0x40040844404084,
                0x2004208A004208,
                0x10190041080202,
                0x108060845042010,
                0x581104180800210,
                0x2112080446200010,
                0x1080820820060210,
                0x3C0808410220200,
                0x4050404440404,
                0x21001420088,
                0x24D0080801082102,
                0x1020A0A020400,
                0x40308200402,
                0x4011002100800,
                0x401484104104005,
                0x801010402020200,
                0x400210C3880100,
                0x404022024108200,
                0x810018200204102,
                0x4002801A02003,
                0x85040820080400,
                0x810102C808880400,
                0xE900410884800,
                0x8002020480840102,
                0x220200865090201,
                0x2010100A02021202,
                0x152048408022401,
                0x20080002081110,
                0x4001001021004000,
                0x800040400A011002,
                0xE4004081011002,
                0x1C004001012080,
                0x8004200962A00220,
                0x8422100208500202,
                0x2000402200300C08,
                0x8646020080080080,
                0x80020A0200100808,
                0x2010004880111000,
                0x623000A080011400,
                0x42008C0340209202,
                0x209188240001000,
                0x400408A884001800,
                0x110400A6080400,
                0x1840060A44020800,
                0x90080104000041,
                0x201011000808101,
                0x1A2208080504F080,
                0x8012020600211212,
                0x500861011240000,
                0x180806108200800,
                0x4000020E01040044,
                0x300000261044000A,
                0x802241102020002,
                0x20906061210001,
                0x5A84841004010310,
                0x4010801011C04,
                0xA010109502200,
                0x4A02012000,
                0x500201010098B028,
                0x8040002811040900,
                0x28000010020204,
                0x6000020202D0240,
                0x8918844842082200,
                0x4010011029020020,
            ],
            rook_magic_numbers: [
                0x8A80104000800020,
                0x140002000100040,
                0x2801880A0017001,
                0x100081001000420,
                0x200020010080420,
                0x3001C0002010008,
                0x8480008002000100,
                0x2080088004402900,
                0x800098204000,
                0x2024401000200040,
                0x100802000801000,
                0x120800800801000,
                0x208808088000400,
                0x2802200800400,
                0x2200800100020080,
                0x801000060821100,
                0x80044006422000,
                0x100808020004000,
                0x12108A0010204200,
                0x140848010000802,
                0x481828014002800,
                0x8094004002004100,
                0x4010040010010802,
                0x20008806104,
                0x100400080208000,
                0x2040002120081000,
                0x21200680100081,
                0x20100080080080,
                0x2000A00200410,
                0x20080800400,
                0x80088400100102,
                0x80004600042881,
                0x4040008040800020,
                0x440003000200801,
                0x4200011004500,
                0x188020010100100,
                0x14800401802800,
                0x2080040080800200,
                0x124080204001001,
                0x200046502000484,
                0x480400080088020,
                0x1000422010034000,
                0x30200100110040,
                0x100021010009,
                0x2002080100110004,
                0x202008004008002,
                0x20020004010100,
                0x2048440040820001,
                0x101002200408200,
                0x40802000401080,
                0x4008142004410100,
                0x2060820C0120200,
                0x1001004080100,
                0x20C020080040080,
                0x2935610830022400,
                0x44440041009200,
                0x280001040802101,
                0x2100190040002085,
                0x80C0084100102001,
                0x4024081001000421,
                0x20030A0244872,
                0x12001008414402,
                0x2006104900A0804,
                0x1004081002402,
            ],
        }
    }

    pub fn generate_magic_index(
        &self,
        occupancy: &Bitboard,
        attack_mask: &Bitboard,
        square: &BoardSquare,
        piece: &Piece,
    ) -> usize {
        match piece {
            Piece::Bishop => {
                (occupancy
                    .bitboard
                    .overflowing_mul(self.magic_number(piece, square))
                    .0
                    >> (64 - attack_mask.count_bits())) as usize
            }
            Piece::Rook => {
                (occupancy
                    .bitboard
                    .overflowing_mul(self.magic_number(piece, square))
                    .0
                    >> (64 - attack_mask.count_bits())) as usize
            }
            _ => panic!("Attempted to generate magic index for non-slider piece"),
        }
    }

    fn magic_number(&self, piece: &Piece, square: &BoardSquare) -> u64 {
        match piece {
            Piece::Bishop => self.bishop_magic_numbers[square.enumeration()],
            Piece::Rook => self.rook_magic_numbers[square.enumeration()],
            _ => panic!("Attempted to access magic number for non-slider piece"),
        }
    }

    // Implementation to generate magic numbers taken from
    // https://www.youtube.com/watch?v=UnEu5GOiSEs&list=PLmN0neTso3Jxh8ZIylk74JpwfiWNI76Cs&index=15
    // NB this seems to take much longer for me - no clue why, must be some problem in the code I can't see
    // Not too important as magic numbers are hard coded anyway
    pub fn _new(random_state: &mut u32) -> Self {
        let mut bishop_magic_numbers: [u64; 64] = [0; 64];
        let mut rook_magic_numbers: [u64; 64] = [0; 64];

        let slider_attack_tables = SliderAttackTables::initialise();

        BoardSquare::iter().for_each(|square| {
            rook_magic_numbers[square.enumeration()] = Self::_generate_magic_number(
                random_state,
                slider_attack_tables.attack_mask(&Piece::Rook, &Side::Either, &square),
                Piece::Rook,
                &square,
            )
        });

        BoardSquare::iter().for_each(|square| {
            bishop_magic_numbers[square.enumeration()] = Self::_generate_magic_number(
                random_state,
                slider_attack_tables.attack_mask(&Piece::Bishop, &Side::Either, &square),
                Piece::Bishop,
                &square,
            )
        });

        Self {
            bishop_magic_numbers,
            rook_magic_numbers,
        }
    }

    fn _generate_magic_number(
        mut random_state: &mut u32,
        attack_mask: Bitboard,
        piece: Piece,
        square: &BoardSquare,
    ) -> u64 {
        let mut occupancies: [Bitboard; 4096] = [Bitboard::new(0); 4096];
        let mut attacks: [Bitboard; 4096] = [Bitboard::new(0); 4096];

        let occupancy_count = attack_mask.count_bits();
        let occupancy_indices = 1 << occupancy_count;

        for i in 0..occupancy_indices {
            occupancies[i] = SliderAttackTables::set_occupancy(i, attack_mask);
            attacks[i] =
                SliderAttackTables::generate_attack_table(&occupancies[i], &piece, &square);
        }

        'outer: loop {
            let magic_number_candidate = Self::_generate_random_u64_integer(&mut random_state)
                & Self::_generate_random_u64_integer(&mut random_state)
                & Self::_generate_random_u64_integer(&mut random_state);

            if (attack_mask
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
                    >> (64 - occupancy_count)) as usize;

                if used_attacks[magic_index].bitboard == 0 {
                    used_attacks[magic_index].bitboard = attacks[i].bitboard;
                } else if used_attacks[magic_index].bitboard != attacks[i].bitboard {
                    continue 'outer;
                }
            }

            return magic_number_candidate;
        }
    }

    fn _generate_random_u64_integer(mut random_state: &mut u32) -> u64 {
        // `& 0xFFFF` operation cuts off first 16 most significant bits from 32 bit integer
        Self::_mutate_random_state(&mut random_state);
        let random_u64_integer_1 = (*random_state & 0xFFFF) as u64;

        Self::_mutate_random_state(&mut random_state);
        let random_u64_integer_2 = (*random_state & 0xFFFF) as u64;

        Self::_mutate_random_state(&mut random_state);
        let random_u64_integer_3 = (*random_state & 0xFFFF) as u64;

        Self::_mutate_random_state(&mut random_state);
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
                .attack_table(
                    &Bitboard::new(0),
                    &Piece::Pawn,
                    &Side::White,
                    &BoardSquare::H3
                )
                .bitboard,
            desired_h3_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(
                    &Bitboard::new(0),
                    &Piece::Pawn,
                    &Side::White,
                    &BoardSquare::F5
                )
                .bitboard,
            desired_f5_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(
                    &Bitboard::new(0),
                    &Piece::Pawn,
                    &Side::White,
                    &BoardSquare::A4
                )
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
                .attack_table(
                    &Bitboard::new(0),
                    &Piece::Pawn,
                    &Side::Black,
                    &BoardSquare::B4
                )
                .bitboard,
            desired_b4_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(
                    &Bitboard::new(0),
                    &Piece::Pawn,
                    &Side::Black,
                    &BoardSquare::H4
                )
                .bitboard,
            desired_h4_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(
                    &Bitboard::new(0),
                    &Piece::Pawn,
                    &Side::Black,
                    &BoardSquare::A5
                )
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
                .attack_table(
                    &Bitboard::new(0),
                    &Piece::Knight,
                    &Side::Either,
                    &BoardSquare::G5
                )
                .bitboard,
            desired_g5_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(
                    &Bitboard::new(0),
                    &Piece::Knight,
                    &Side::Either,
                    &BoardSquare::E2
                )
                .bitboard,
            desired_e2_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(
                    &Bitboard::new(0),
                    &Piece::Knight,
                    &Side::Either,
                    &BoardSquare::F4
                )
                .bitboard,
            desired_f4_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(
                    &Bitboard::new(0),
                    &Piece::Knight,
                    &Side::Either,
                    &BoardSquare::B4
                )
                .bitboard,
            desired_b4_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(
                    &Bitboard::new(0),
                    &Piece::Knight,
                    &Side::Either,
                    &BoardSquare::A4
                )
                .bitboard,
            desired_a4_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(
                    &Bitboard::new(0),
                    &Piece::Knight,
                    &Side::Either,
                    &BoardSquare::H8
                )
                .bitboard,
            desired_h8_attack_table
        );
    }

    #[test]
    fn attack_masks_bishop() {
        let attack_tables = SliderAttackTables::initialise();

        let desired_a5_attack_mask = u64::pow(2, BoardSquare::B6 as u32)
            + u64::pow(2, BoardSquare::C7 as u32)
            + u64::pow(2, BoardSquare::B4 as u32)
            + u64::pow(2, BoardSquare::C3 as u32)
            + u64::pow(2, BoardSquare::D2 as u32);
        let desired_g7_attack_mask = u64::pow(2, BoardSquare::F6 as u32)
            + u64::pow(2, BoardSquare::E5 as u32)
            + u64::pow(2, BoardSquare::D4 as u32)
            + u64::pow(2, BoardSquare::C3 as u32)
            + u64::pow(2, BoardSquare::B2 as u32);
        let desired_d6_attack_mask = u64::pow(2, BoardSquare::C7 as u32)
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
            desired_a5_attack_mask
        );
        assert_eq!(
            attack_tables
                .attack_mask(&Piece::Bishop, &Side::Either, &BoardSquare::G7)
                .bitboard,
            desired_g7_attack_mask
        );
        assert_eq!(
            attack_tables
                .attack_mask(&Piece::Bishop, &Side::Either, &BoardSquare::D6)
                .bitboard,
            desired_d6_attack_mask
        );
    }

    #[test]
    fn attack_tables_bishop() {
        let attack_tables = SliderAttackTables::initialise();

        let mut board = Bitboard::new(0);

        let desired_attack_table = u64::pow(2, BoardSquare::A7 as u32)
            + u64::pow(2, BoardSquare::B6 as u32)
            + u64::pow(2, BoardSquare::C5 as u32)
            + u64::pow(2, BoardSquare::E5 as u32)
            + u64::pow(2, BoardSquare::F6 as u32)
            + u64::pow(2, BoardSquare::G7 as u32)
            + u64::pow(2, BoardSquare::H8 as u32)
            + u64::pow(2, BoardSquare::C3 as u32)
            + u64::pow(2, BoardSquare::B2 as u32)
            + u64::pow(2, BoardSquare::A1 as u32)
            + u64::pow(2, BoardSquare::E3 as u32)
            + u64::pow(2, BoardSquare::F2 as u32)
            + u64::pow(2, BoardSquare::G1 as u32);

        assert_eq!(
            attack_tables
                .attack_table(&board, &Piece::Bishop, &Side::Either, &BoardSquare::D4)
                .bitboard,
            desired_attack_table
        );

        board.set_bit(&BoardSquare::C5);

        let blocked_desired_attack_table = desired_attack_table
            - u64::pow(2, BoardSquare::A7 as u32)
            - u64::pow(2, BoardSquare::B6 as u32);

        assert_eq!(
            attack_tables
                .attack_table(&board, &Piece::Bishop, &Side::Either, &BoardSquare::D4)
                .bitboard,
            blocked_desired_attack_table
        );

        board.set_bit(&BoardSquare::F2);

        let blocked_desired_attack_table =
            blocked_desired_attack_table - u64::pow(2, BoardSquare::G1 as u32);

        assert_eq!(
            attack_tables
                .attack_table(&board, &Piece::Bishop, &Side::Either, &BoardSquare::D4)
                .bitboard,
            blocked_desired_attack_table
        );

        board.set_bit(&BoardSquare::G7);

        let blocked_desired_attack_table =
            blocked_desired_attack_table - u64::pow(2, BoardSquare::H8 as u32);

        assert_eq!(
            attack_tables
                .attack_table(&board, &Piece::Bishop, &Side::Either, &BoardSquare::D4)
                .bitboard,
            blocked_desired_attack_table
        );

        board.pop_bit(&BoardSquare::G7);
        board.set_bit(&BoardSquare::H8);

        let blocked_desired_attack_table =
            blocked_desired_attack_table + u64::pow(2, BoardSquare::H8 as u32);

        assert_eq!(
            attack_tables
                .attack_table(&board, &Piece::Bishop, &Side::Either, &BoardSquare::D4)
                .bitboard,
            blocked_desired_attack_table
        );

        board.set_bit(&BoardSquare::G8);

        assert_eq!(
            attack_tables
                .attack_table(&board, &Piece::Bishop, &Side::Either, &BoardSquare::D4)
                .bitboard,
            blocked_desired_attack_table
        );
    }

    #[test]
    fn attack_masks_rook() {
        let attack_tables = SliderAttackTables::initialise();

        let desired_d5_attack_mask = u64::pow(2, BoardSquare::D7 as u32)
            + u64::pow(2, BoardSquare::D6 as u32)
            + u64::pow(2, BoardSquare::B5 as u32)
            + u64::pow(2, BoardSquare::C5 as u32)
            + u64::pow(2, BoardSquare::E5 as u32)
            + u64::pow(2, BoardSquare::F5 as u32)
            + u64::pow(2, BoardSquare::G5 as u32)
            + u64::pow(2, BoardSquare::D4 as u32)
            + u64::pow(2, BoardSquare::D3 as u32)
            + u64::pow(2, BoardSquare::D2 as u32);
        let desired_b3_attack_mask = u64::pow(2, BoardSquare::B7 as u32)
            + u64::pow(2, BoardSquare::B6 as u32)
            + u64::pow(2, BoardSquare::B5 as u32)
            + u64::pow(2, BoardSquare::B4 as u32)
            + u64::pow(2, BoardSquare::B2 as u32)
            + u64::pow(2, BoardSquare::C3 as u32)
            + u64::pow(2, BoardSquare::D3 as u32)
            + u64::pow(2, BoardSquare::E3 as u32)
            + u64::pow(2, BoardSquare::F3 as u32)
            + u64::pow(2, BoardSquare::G3 as u32);
        let desired_e1_attack_mask = u64::pow(2, BoardSquare::E7 as u32)
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
            desired_d5_attack_mask
        );
        assert_eq!(
            attack_tables
                .attack_mask(&Piece::Rook, &Side::Either, &BoardSquare::B3)
                .bitboard,
            desired_b3_attack_mask
        );
        assert_eq!(
            attack_tables
                .attack_mask(&Piece::Rook, &Side::Either, &BoardSquare::E1)
                .bitboard,
            desired_e1_attack_mask
        );
    }

    #[test]
    fn attack_tables_rook() {
        let attack_tables = SliderAttackTables::initialise();

        let mut board = Bitboard::new(0);

        let desired_attack_table = u64::pow(2, BoardSquare::E8 as u32)
            + u64::pow(2, BoardSquare::E7 as u32)
            + u64::pow(2, BoardSquare::E6 as u32)
            + u64::pow(2, BoardSquare::E4 as u32)
            + u64::pow(2, BoardSquare::E3 as u32)
            + u64::pow(2, BoardSquare::E2 as u32)
            + u64::pow(2, BoardSquare::E1 as u32)
            + u64::pow(2, BoardSquare::A5 as u32)
            + u64::pow(2, BoardSquare::B5 as u32)
            + u64::pow(2, BoardSquare::C5 as u32)
            + u64::pow(2, BoardSquare::D5 as u32)
            + u64::pow(2, BoardSquare::F5 as u32)
            + u64::pow(2, BoardSquare::G5 as u32)
            + u64::pow(2, BoardSquare::H5 as u32);

        assert_eq!(
            attack_tables
                .attack_table(&board, &Piece::Rook, &Side::Either, &BoardSquare::E5)
                .bitboard,
            desired_attack_table
        );

        board.set_bit(&BoardSquare::E7);

        let blocked_desired_attack_table =
            desired_attack_table - u64::pow(2, BoardSquare::E8 as u32);

        assert_eq!(
            attack_tables
                .attack_table(&board, &Piece::Rook, &Side::Either, &BoardSquare::E5)
                .bitboard,
            blocked_desired_attack_table
        );

        board.set_bit(&BoardSquare::E2);

        let blocked_desired_attack_table =
            blocked_desired_attack_table - u64::pow(2, BoardSquare::E1 as u32);

        assert_eq!(
            attack_tables
                .attack_table(&board, &Piece::Rook, &Side::Either, &BoardSquare::E5)
                .bitboard,
            blocked_desired_attack_table
        );

        board.set_bit(&BoardSquare::C5);

        let blocked_desired_attack_table = blocked_desired_attack_table
            - u64::pow(2, BoardSquare::A5 as u32)
            - u64::pow(2, BoardSquare::B5 as u32);

        assert_eq!(
            attack_tables
                .attack_table(&board, &Piece::Rook, &Side::Either, &BoardSquare::E5)
                .bitboard,
            blocked_desired_attack_table
        );

        board.set_bit(&BoardSquare::G5);

        let blocked_desired_attack_table =
            blocked_desired_attack_table - u64::pow(2, BoardSquare::H5 as u32);

        assert_eq!(
            attack_tables
                .attack_table(&board, &Piece::Rook, &Side::Either, &BoardSquare::E5)
                .bitboard,
            blocked_desired_attack_table
        );

        board.pop_bit(&BoardSquare::G5);
        board.set_bit(&BoardSquare::H5);

        let blocked_desired_attack_table =
            blocked_desired_attack_table + u64::pow(2, BoardSquare::H5 as u32);

        assert_eq!(
            attack_tables
                .attack_table(&board, &Piece::Rook, &Side::Either, &BoardSquare::E5)
                .bitboard,
            blocked_desired_attack_table
        );

        board.set_bit(&BoardSquare::C8);

        assert_eq!(
            attack_tables
                .attack_table(&board, &Piece::Rook, &Side::Either, &BoardSquare::E5)
                .bitboard,
            blocked_desired_attack_table
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
                .attack_table(
                    &Bitboard::new(0),
                    &Piece::King,
                    &Side::Either,
                    &BoardSquare::B2
                )
                .bitboard,
            desired_b2_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(
                    &Bitboard::new(0),
                    &Piece::King,
                    &Side::Either,
                    &BoardSquare::A1
                )
                .bitboard,
            desired_a1_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(
                    &Bitboard::new(0),
                    &Piece::King,
                    &Side::Either,
                    &BoardSquare::H4
                )
                .bitboard,
            desired_h4_attack_table
        );
    }
}
