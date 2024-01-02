use super::game::{Bitboard, Piece, Side, Square};
use crate::random;
use strum::IntoEnumIterator;

type MagicNumber = u64;

// Magic numbers generated with random_state = 1_804_289_383
const MAGIC_NUMBERS: MagicNumbers = MagicNumbers {
    bishop_magic_numbers: [
        0x0040_0408_4440_4084,
        0x0020_0420_8A00_4208,
        0x0010_1900_4108_0202,
        0x0108_0608_4504_2010,
        0x0581_1041_8080_0210,
        0x2112_0804_4620_0010,
        0x1080_8208_2006_0210,
        0x03C0_8084_1022_0200,
        0x0004_0504_0444_0404,
        0x0000_0210_0142_0088,
        0x24D0_0808_0108_2102,
        0x0001_020A_0A02_0400,
        0x0000_0403_0820_0402,
        0x0004_0110_0210_0800,
        0x0401_4841_0410_4005,
        0x0801_0104_0202_0200,
        0x0040_0210_C388_0100,
        0x0404_0220_2410_8200,
        0x0810_0182_0020_4102,
        0x0004_0028_01A0_2003,
        0x0085_0408_2008_0400,
        0x8101_02C8_0888_0400,
        0x000E_9004_1088_4800,
        0x8002_0204_8084_0102,
        0x0220_2008_6509_0201,
        0x2010_100A_0202_1202,
        0x0152_0484_0802_2401,
        0x0020_0800_0208_1110,
        0x4001_0010_2100_4000,
        0x8000_4040_0A01_1002,
        0x00E4_0040_8101_1002,
        0x001C_0040_0101_2080,
        0x8004_2009_62A0_0220,
        0x8422_1002_0850_0202,
        0x2000_4022_0030_0c08,
        0x8646_0200_8008_0080,
        0x8002_0A02_0010_0808,
        0x2010_0048_8011_1000,
        0x6230_00A0_8001_1400,
        0x4200_8C03_4020_9202,
        0x0209_1882_4000_1000,
        0x4004_08A8_8400_1800,
        0x0011_0400_A608_0400,
        0x1840_060A_4402_0800,
        0x0090_0801_0400_0041,
        0x0201_0110_0080_8101,
        0x1A22_0808_0504_F080,
        0x8012_0206_0021_1212,
        0x0500_8610_1124_0000,
        0x0180_8061_0820_0800,
        0x4000_020E_0104_0044,
        0x3000_0026_1044_000A,
        0x0802_2411_0202_0002,
        0x0020_9060_6121_0001,
        0x5A84_8410_0401_0310,
        0x0004_0108_0101_1C04,
        0x000A_0101_0950_2200,
        0x0000_004A_0201_2000,
        0x5002_0101_0098_B028,
        0x8040_0028_1104_0900,
        0x0028_0000_1002_0204,
        0x0600_0020_202D_0240,
        0x8918_8448_4208_2200,
        0x4010_0110_2902_0020,
    ],
    rook_magic_numbers: [
        0x8A80_1040_0080_0020,
        0x0140_0020_0010_0040,
        0x0280_1880_A001_7001,
        0x0100_0810_0100_0420,
        0x0200_0200_1008_0420,
        0x0300_1C00_0201_0008,
        0x8480_0080_0200_0100,
        0x2080_0880_0440_2900,
        0x0000_8000_9820_4000,
        0x2024_4010_0020_0040,
        0x0100_8020_0080_1000,
        0x0120_8008_0080_1000,
        0x0208_8080_8800_0400,
        0x0002_8022_0080_0400,
        0x2200_8001_0002_0080,
        0x0801_0000_6082_1100,
        0x0080_0440_0642_2000,
        0x0100_8080_2000_4000,
        0x1210_8A00_1020_4200,
        0x0140_8480_1000_0802,
        0x0481_8280_1400_2800,
        0x8094_0040_0200_4100,
        0x4010_0400_1001_0802,
        0x0000_0200_0880_6104,
        0x0100_4000_8020_8000,
        0x2040_0021_2008_1000,
        0x0021_2006_8010_0081,
        0x0020_1000_8008_0080,
        0x0002_000A_0020_0410,
        0x0000_0200_8080_0400,
        0x0080_0884_0010_0102,
        0x0080_0046_0004_2881,
        0x4040_0080_4080_0020,
        0x0440_0030_0020_0801,
        0x0004_2000_1100_4500,
        0x0188_0200_1010_0100,
        0x0014_8004_0180_2800,
        0x2080_0400_8080_0200,
        0x0124_0802_0400_1001,
        0x0200_0465_0200_0484,
        0x0480_4000_8008_8020,
        0x1000_4220_1003_4000,
        0x0030_2001_0011_0040,
        0x0000_1000_2101_0009,
        0x2002_0801_0011_0004,
        0x0202_0080_0400_8002,
        0x0020_0200_0401_0100,
        0x2048_4400_4082_0001,
        0x0101_0022_0040_8200,
        0x0040_8020_0040_1080,
        0x4008_1420_0441_0100,
        0x0206_0820_C012_0200,
        0x0001_0010_0408_0100,
        0x020C_0200_8004_0080,
        0x2935_6108_3002_2400,
        0x0044_4400_4100_9200,
        0x0280_0010_4080_2101,
        0x2100_1900_4000_2085,
        0x80C0_0841_0010_2001,
        0x4024_0810_0100_0421,
        0x0002_0030_A024_4872,
        0x0012_0010_0841_4402,
        0x0200_6104_900A_0804,
        0x0001_0040_8100_2402,
    ],
};

const BISHOP_MAX_OCCUPANCY_INDEX_MAX: usize = 512;
const ROOK_OCCUPANCY_INDEX_MAX: usize = 4096;

#[rustfmt::skip]
const BISHOP_ATTACK_MASK_BIT_COUNT: [u8; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6,
    5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5, 
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5,
    6, 5, 5, 5, 5, 5, 5, 6,
];
#[rustfmt::skip]
const ROOK_ATTACK_MASK_BIT_COUNT: [u8; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    12, 11, 11, 11, 11, 11, 11, 12,
];

#[derive(Clone)]
pub struct AttackTables {
    leaper_attack_tables: LeaperAttackTables,
    slider_attack_tables: SliderAttackTables,
}

impl AttackTables {
    pub fn initialise() -> Self {
        Self {
            leaper_attack_tables: LeaperAttackTables::initialise(),
            slider_attack_tables: SliderAttackTables::initialise(),
        }
    }

    pub fn attack_table(
        &self,
        board: Bitboard,
        piece: Piece,
        side: Side,
        square: Square,
    ) -> Bitboard {
        match piece {
            Piece::Pawn => match side {
                Side::White => self.leaper_attack_tables.white_pawn_attack_tables[square as usize],
                Side::Black => self.leaper_attack_tables.black_pawn_attack_tables[square as usize],
            },
            Piece::Knight => self.leaper_attack_tables.knight_attack_tables[square as usize],
            Piece::Bishop => {
                let attack_mask = self
                    .slider_attack_tables
                    .attack_mask(SliderPiece::Bishop, square);
                let magic_index =
                    MAGIC_NUMBERS.get_magic_index(attack_mask, board, SliderPiece::Bishop, square);

                self.slider_attack_tables.bishop_attack_tables[square as usize][magic_index]
            }
            Piece::Rook => {
                let attack_mask = self
                    .slider_attack_tables
                    .attack_mask(SliderPiece::Rook, square);
                let magic_index =
                    MAGIC_NUMBERS.get_magic_index(attack_mask, board, SliderPiece::Rook, square);

                self.slider_attack_tables.rook_attack_tables[square as usize][magic_index]
            }
            Piece::Queen => {
                let bishop_attack_mask = self
                    .slider_attack_tables
                    .attack_mask(SliderPiece::Bishop, square);
                let rook_attack_mask = self
                    .slider_attack_tables
                    .attack_mask(SliderPiece::Rook, square);
                let bishop_magic_index = MAGIC_NUMBERS.get_magic_index(
                    bishop_attack_mask,
                    board,
                    SliderPiece::Bishop,
                    square,
                );
                let rook_magic_index = MAGIC_NUMBERS.get_magic_index(
                    rook_attack_mask,
                    board,
                    SliderPiece::Rook,
                    square,
                );

                self.slider_attack_tables.bishop_attack_tables[square as usize][bishop_magic_index]
                    | self.slider_attack_tables.rook_attack_tables[square as usize]
                        [rook_magic_index]
            }
            Piece::King => self.leaper_attack_tables.king_attack_tables[square as usize],
        }
    }
}

#[derive(Clone)]
struct LeaperAttackTables {
    white_pawn_attack_tables: [Bitboard; 64],
    black_pawn_attack_tables: [Bitboard; 64],
    knight_attack_tables: [Bitboard; 64],
    king_attack_tables: [Bitboard; 64],
}

impl LeaperAttackTables {
    fn initialise() -> Self {
        Self {
            white_pawn_attack_tables: Self::generate_attack_tables(LeaperPiece::Pawn, Side::White),
            black_pawn_attack_tables: Self::generate_attack_tables(LeaperPiece::Pawn, Side::Black),
            knight_attack_tables: Self::generate_attack_tables(LeaperPiece::Knight, Side::White),
            king_attack_tables: Self::generate_attack_tables(LeaperPiece::King, Side::White),
        }
    }

    fn generate_attack_tables(piece: LeaperPiece, side: Side) -> [Bitboard; 64] {
        let not_a_file = Bitboard::new(0xFEFE_FEFE_FEFE_FEFE);
        let not_h_file = Bitboard::new(0x7F7F_7F7F_7F7F_7F7F);
        let not_ab_file = Bitboard::new(0xFCFC_FCFC_FCFC_FCFC);
        let not_gh_file = Bitboard::new(0x3F3F_3F3F_3F3F_3F3F);

        let mut attack_tables = [Bitboard::new(0); 64];

        for square in Square::iter() {
            let bitboard = Bitboard::from_square(square);
            let mut attack_table = Bitboard::new(0);

            attack_tables[square as usize] = match piece {
                LeaperPiece::Pawn => {
                    match side {
                        Side::White => {
                            attack_table |= (bitboard >> 7u64) & not_a_file;
                            attack_table |= (bitboard >> 9u64) & not_h_file;
                        }
                        Side::Black => {
                            attack_table |= (bitboard << 7u64) & not_h_file;
                            attack_table |= (bitboard << 9u64) & not_a_file;
                        }
                    }

                    attack_table
                }
                LeaperPiece::Knight => {
                    attack_table |= (bitboard >> 6u64) & not_ab_file;
                    attack_table |= (bitboard >> 10u64) & not_gh_file;
                    attack_table |= (bitboard >> 15u64) & not_a_file;
                    attack_table |= (bitboard >> 17u64) & not_h_file;
                    attack_table |= (bitboard << 6u64) & not_gh_file;
                    attack_table |= (bitboard << 10u64) & not_ab_file;
                    attack_table |= (bitboard << 15u64) & not_h_file;
                    attack_table |= (bitboard << 17u64) & not_a_file;

                    attack_table
                }
                LeaperPiece::King => {
                    attack_table |= (bitboard >> 1u64) & not_h_file;
                    attack_table |= (bitboard >> 7u64) & not_a_file;
                    attack_table |= bitboard >> 8u64;
                    attack_table |= (bitboard >> 9u64) & not_h_file;
                    attack_table |= (bitboard << 1u64) & not_a_file;
                    attack_table |= (bitboard << 7u64) & not_h_file;
                    attack_table |= bitboard << 8u64;
                    attack_table |= (bitboard << 9u64) & not_a_file;

                    attack_table
                }
            }
        }

        attack_tables
    }
}

#[derive(Clone, Copy)]
enum LeaperPiece {
    Pawn = 0,
    Knight = 1,
    King = 5,
}

#[derive(Clone)]
struct SliderAttackTables {
    bishop_attack_masks: [Bitboard; 64],
    rook_attack_masks: [Bitboard; 64],
    bishop_attack_tables: Vec<[Bitboard; BISHOP_MAX_OCCUPANCY_INDEX_MAX]>,
    rook_attack_tables: Vec<[Bitboard; ROOK_OCCUPANCY_INDEX_MAX]>,
}

impl SliderAttackTables {
    fn initialise() -> Self {
        let bishop_attack_masks = Self::generate_attack_masks(SliderPiece::Bishop);
        let rook_attack_masks = Self::generate_attack_masks(SliderPiece::Rook);
        let mut bishop_attack_tables = vec![[Bitboard::new(0); BISHOP_MAX_OCCUPANCY_INDEX_MAX]; 64];
        let mut rook_attack_tables = vec![[Bitboard::new(0); ROOK_OCCUPANCY_INDEX_MAX]; 64];

        for square in Square::iter() {
            let bishop_occupancy_indices = 1 << BISHOP_ATTACK_MASK_BIT_COUNT[square as usize];

            for index in 0..bishop_occupancy_indices {
                let occupancy = Self::set_occupancy(index, bishop_attack_masks[square as usize]);
                let magic_index = MAGIC_NUMBERS.get_magic_index(
                    bishop_attack_masks[square as usize],
                    occupancy,
                    SliderPiece::Bishop,
                    square,
                );
                bishop_attack_tables[square as usize][magic_index] =
                    Self::generate_attack_table(occupancy, SliderPiece::Bishop, square);
            }

            let rook_occupancy_indices = 1 << ROOK_ATTACK_MASK_BIT_COUNT[square as usize];

            for index in 0..rook_occupancy_indices {
                let occupancy = Self::set_occupancy(index, rook_attack_masks[square as usize]);
                let magic_index = MAGIC_NUMBERS.get_magic_index(
                    rook_attack_masks[square as usize],
                    occupancy,
                    SliderPiece::Rook,
                    square,
                );
                rook_attack_tables[square as usize][magic_index] =
                    Self::generate_attack_table(occupancy, SliderPiece::Rook, square);
            }
        }

        Self {
            bishop_attack_masks,
            rook_attack_masks,
            bishop_attack_tables,
            rook_attack_tables,
        }
    }

    fn generate_attack_masks(piece: SliderPiece) -> [Bitboard; 64] {
        let mut attack_masks = [Bitboard::new(0); 64];

        for square in Square::iter() {
            let mut attack_mask = Bitboard::new(0);
            let piece_rank = square.rank();
            let piece_file = square.file();

            match piece {
                SliderPiece::Bishop => {
                    for (rank, file) in ((piece_rank + 1)..7).zip((piece_file + 1)..7) {
                        attack_mask.set_bit(Square::from_rank_file(rank, file));
                    }

                    for (rank, file) in ((1..piece_rank).rev()).zip((piece_file + 1)..7) {
                        attack_mask.set_bit(Square::from_rank_file(rank, file));
                    }

                    for (rank, file) in ((piece_rank + 1)..7).zip((1..piece_file).rev()) {
                        attack_mask.set_bit(Square::from_rank_file(rank, file));
                    }

                    for (rank, file) in ((1..piece_rank).rev()).zip((1..piece_file).rev()) {
                        attack_mask.set_bit(Square::from_rank_file(rank, file));
                    }

                    attack_masks[square as usize] = attack_mask;
                }
                SliderPiece::Rook => {
                    for rank in (piece_rank + 1)..7 {
                        attack_mask.set_bit(Square::from_rank_file(rank, piece_file));
                    }

                    for rank in (1..piece_rank).rev() {
                        attack_mask.set_bit(Square::from_rank_file(rank, piece_file));
                    }

                    for file in (piece_file + 1)..7 {
                        attack_mask.set_bit(Square::from_rank_file(piece_rank, file));
                    }

                    for file in (1..piece_file).rev() {
                        attack_mask.set_bit(Square::from_rank_file(piece_rank, file));
                    }

                    attack_masks[square as usize] = attack_mask;
                }
            }
        }

        attack_masks
    }

    fn generate_attack_table(board: Bitboard, piece: SliderPiece, square: Square) -> Bitboard {
        let mut attack_table = Bitboard::new(0);
        let piece_rank = square.rank();
        let piece_file = square.file();

        match piece {
            SliderPiece::Bishop => {
                for (rank, file) in ((piece_rank + 1)..8).zip((piece_file + 1)..8) {
                    let square = Square::from_rank_file(rank, file);
                    attack_table.set_bit(square);

                    if board.bit_occupied(square) {
                        break;
                    }
                }

                for (rank, file) in ((0..piece_rank).rev()).zip((piece_file + 1)..8) {
                    let square = Square::from_rank_file(rank, file);
                    attack_table.set_bit(square);

                    if board.bit_occupied(square) {
                        break;
                    }
                }

                for (rank, file) in ((piece_rank + 1)..8).zip((0..piece_file).rev()) {
                    let square = Square::from_rank_file(rank, file);
                    attack_table.set_bit(square);

                    if board.bit_occupied(square) {
                        break;
                    }
                }

                for (rank, file) in ((0..piece_rank).rev()).zip((0..piece_file).rev()) {
                    let square = Square::from_rank_file(rank, file);
                    attack_table.set_bit(square);

                    if board.bit_occupied(square) {
                        break;
                    }
                }
            }
            SliderPiece::Rook => {
                for rank in (piece_rank + 1)..8 {
                    let square = Square::from_rank_file(rank, piece_file);
                    attack_table.set_bit(square);

                    if board.bit_occupied(square) {
                        break;
                    }
                }

                for rank in (0..piece_rank).rev() {
                    let square = Square::from_rank_file(rank, piece_file);
                    attack_table.set_bit(square);

                    if board.bit_occupied(square) {
                        break;
                    }
                }

                for file in (piece_file + 1)..8 {
                    let square = Square::from_rank_file(piece_rank, file);
                    attack_table.set_bit(square);

                    if board.bit_occupied(square) {
                        break;
                    }
                }

                for file in (0..piece_file).rev() {
                    let square = Square::from_rank_file(piece_rank, file);
                    attack_table.set_bit(square);

                    if board.bit_occupied(square) {
                        break;
                    }
                }
            }
        }

        attack_table
    }

    fn set_occupancy(index: usize, mut attack_mask: Bitboard) -> Bitboard {
        let mut occupancy = Bitboard::new(0);
        let mut count = 0;

        while let Some(lsb_square) = attack_mask.get_lsb_square() {
            if index & (1 << count) != 0 {
                occupancy.set_bit(lsb_square);
            }

            attack_mask.pop_bit(lsb_square);
            count += 1;
        }

        occupancy
    }

    fn attack_mask(&self, piece: SliderPiece, square: Square) -> Bitboard {
        match piece {
            SliderPiece::Bishop => self.bishop_attack_masks[square as usize],
            SliderPiece::Rook => self.rook_attack_masks[square as usize],
        }
    }
}

#[derive(Clone, Copy)]
enum SliderPiece {
    Bishop = 2,
    Rook = 3,
}

#[derive(Debug, PartialEq)]
struct MagicNumbers {
    bishop_magic_numbers: [MagicNumber; 64],
    rook_magic_numbers: [MagicNumber; 64],
}

impl MagicNumbers {
    fn get_magic_index(
        &self,
        attack_mask: Bitboard,
        board: Bitboard,
        piece: SliderPiece,
        square: Square,
    ) -> usize {
        let bit_count = match piece {
            SliderPiece::Bishop => BISHOP_ATTACK_MASK_BIT_COUNT[square as usize],
            SliderPiece::Rook => ROOK_ATTACK_MASK_BIT_COUNT[square as usize],
        };
        let magic_index = (board & attack_mask)
            .value()
            .overflowing_mul(self.magic_number(piece, square))
            .0
            >> (64 - bit_count);

        magic_index as usize
    }

    fn magic_number(&self, piece: SliderPiece, square: Square) -> MagicNumber {
        match piece {
            SliderPiece::Bishop => self.bishop_magic_numbers[square as usize],
            SliderPiece::Rook => self.rook_magic_numbers[square as usize],
        }
    }

    pub fn _initialise(random_state: &mut u32) -> Self {
        let mut rook_magic_numbers = [0; 64];
        let mut bishop_magic_numbers = [0; 64];
        let rook_attack_masks = SliderAttackTables::generate_attack_masks(SliderPiece::Rook);
        let bishop_attack_masks = SliderAttackTables::generate_attack_masks(SliderPiece::Bishop);

        for square in Square::iter() {
            rook_magic_numbers[square as usize] = Self::_generate_magic_number(
                random_state,
                rook_attack_masks[square as usize],
                SliderPiece::Rook,
                square,
            )
        }

        for square in Square::iter() {
            bishop_magic_numbers[square as usize] = Self::_generate_magic_number(
                random_state,
                bishop_attack_masks[square as usize],
                SliderPiece::Bishop,
                square,
            )
        }

        Self {
            bishop_magic_numbers,
            rook_magic_numbers,
        }
    }

    fn _generate_magic_number(
        random_state: &mut u32,
        attack_mask: Bitboard,
        piece: SliderPiece,
        square: Square,
    ) -> MagicNumber {
        let mut occupancies = [Bitboard::new(0); ROOK_OCCUPANCY_INDEX_MAX];
        let mut attacks = [Bitboard::new(0); ROOK_OCCUPANCY_INDEX_MAX];
        let occupancy_count = match piece {
            SliderPiece::Bishop => BISHOP_ATTACK_MASK_BIT_COUNT[square as usize],
            SliderPiece::Rook => ROOK_ATTACK_MASK_BIT_COUNT[square as usize],
        };
        let occupancy_indices = 1 << occupancy_count;

        for index in 0..occupancy_indices {
            occupancies[index] = SliderAttackTables::set_occupancy(index, attack_mask);
            attacks[index] =
                SliderAttackTables::generate_attack_table(occupancies[index], piece, square);
        }

        'outer: loop {
            let magic_number_candidate = random::_generate_random_u64(random_state)
                & random::_generate_random_u64(random_state)
                & random::_generate_random_u64(random_state);
            let inappropriate_candidate = (attack_mask
                .value()
                .overflowing_mul(magic_number_candidate)
                .0
                & 0xFF00_0000_0000_0000)
                .count_ones()
                < 6;

            if inappropriate_candidate {
                continue;
            };

            let mut used_attacks = [Bitboard::new(0); ROOK_OCCUPANCY_INDEX_MAX];

            for index in 0..occupancy_indices {
                let magic_index = ((occupancies[index]
                    .value()
                    .overflowing_mul(magic_number_candidate)
                    .0)
                    >> (64 - occupancy_count)) as usize;

                if used_attacks[magic_index] == 0u64 {
                    used_attacks[magic_index] = attacks[index];
                } else if used_attacks[magic_index] != attacks[index] {
                    continue 'outer;
                }
            }

            return magic_number_candidate;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attack_tables_white_pawn() {
        let attack_tables = AttackTables::initialise();

        let desired_h3_attack_table = u64::pow(2, Square::G4 as u32);
        let desired_f5_attack_table =
            u64::pow(2, Square::E6 as u32) + u64::pow(2, Square::G6 as u32);
        let desired_a4_attack_table = u64::pow(2, Square::B5 as u32);

        assert_eq!(
            attack_tables
                .attack_table(Bitboard::new(0), Piece::Pawn, Side::White, Square::H3)
                .value(),
            desired_h3_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(Bitboard::new(0), Piece::Pawn, Side::White, Square::F5)
                .value(),
            desired_f5_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(Bitboard::new(0), Piece::Pawn, Side::White, Square::A4)
                .value(),
            desired_a4_attack_table
        );
    }

    #[test]
    fn attack_tables_black_pawn() {
        let attack_tables = AttackTables::initialise();

        let desired_b4_attack_table =
            u64::pow(2, Square::A3 as u32) + u64::pow(2, Square::C3 as u32);
        let desired_h4_attack_table = u64::pow(2, Square::G3 as u32);
        let desired_a5_attack_table = u64::pow(2, Square::B4 as u32);

        assert_eq!(
            attack_tables
                .attack_table(Bitboard::new(0), Piece::Pawn, Side::Black, Square::B4)
                .value(),
            desired_b4_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(Bitboard::new(0), Piece::Pawn, Side::Black, Square::H4)
                .value(),
            desired_h4_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(Bitboard::new(0), Piece::Pawn, Side::Black, Square::A5)
                .value(),
            desired_a5_attack_table
        );
    }

    #[test]
    fn attack_tables_knight() {
        let attack_tables = AttackTables::initialise();

        let desired_g5_attack_table = u64::pow(2, Square::F7 as u32)
            + u64::pow(2, Square::H7 as u32)
            + u64::pow(2, Square::E6 as u32)
            + u64::pow(2, Square::E4 as u32)
            + u64::pow(2, Square::F3 as u32)
            + u64::pow(2, Square::H3 as u32);
        let desired_e2_attack_table = u64::pow(2, Square::D4 as u32)
            + u64::pow(2, Square::F4 as u32)
            + u64::pow(2, Square::C3 as u32)
            + u64::pow(2, Square::G3 as u32)
            + u64::pow(2, Square::C1 as u32)
            + u64::pow(2, Square::G1 as u32);
        let desired_f4_attack_table = u64::pow(2, Square::E6 as u32)
            + u64::pow(2, Square::G6 as u32)
            + u64::pow(2, Square::D5 as u32)
            + u64::pow(2, Square::H5 as u32)
            + u64::pow(2, Square::D3 as u32)
            + u64::pow(2, Square::H3 as u32)
            + u64::pow(2, Square::E2 as u32)
            + u64::pow(2, Square::G2 as u32);
        let desired_b4_attack_table = u64::pow(2, Square::A6 as u32)
            + u64::pow(2, Square::C6 as u32)
            + u64::pow(2, Square::D5 as u32)
            + u64::pow(2, Square::D3 as u32)
            + u64::pow(2, Square::A2 as u32)
            + u64::pow(2, Square::C2 as u32);
        let desired_a4_attack_table = u64::pow(2, Square::B6 as u32)
            + u64::pow(2, Square::C5 as u32)
            + u64::pow(2, Square::C3 as u32)
            + u64::pow(2, Square::B2 as u32);
        let desired_h8_attack_table =
            u64::pow(2, Square::F7 as u32) + u64::pow(2, Square::G6 as u32);

        assert_eq!(
            attack_tables
                .attack_table(Bitboard::new(0), Piece::Knight, Side::White, Square::G5)
                .value(),
            desired_g5_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(Bitboard::new(0), Piece::Knight, Side::White, Square::E2)
                .value(),
            desired_e2_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(Bitboard::new(0), Piece::Knight, Side::White, Square::F4)
                .value(),
            desired_f4_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(Bitboard::new(0), Piece::Knight, Side::White, Square::B4)
                .value(),
            desired_b4_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(Bitboard::new(0), Piece::Knight, Side::White, Square::A4)
                .value(),
            desired_a4_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(Bitboard::new(0), Piece::Knight, Side::White, Square::H8)
                .value(),
            desired_h8_attack_table
        );
    }

    #[test]
    fn attack_masks_bishop() {
        let attack_tables = AttackTables::initialise();

        let desired_a5_attack_mask = u64::pow(2, Square::B6 as u32)
            + u64::pow(2, Square::C7 as u32)
            + u64::pow(2, Square::B4 as u32)
            + u64::pow(2, Square::C3 as u32)
            + u64::pow(2, Square::D2 as u32);
        let desired_g7_attack_mask = u64::pow(2, Square::F6 as u32)
            + u64::pow(2, Square::E5 as u32)
            + u64::pow(2, Square::D4 as u32)
            + u64::pow(2, Square::C3 as u32)
            + u64::pow(2, Square::B2 as u32);
        let desired_d6_attack_mask = u64::pow(2, Square::C7 as u32)
            + u64::pow(2, Square::E7 as u32)
            + u64::pow(2, Square::C5 as u32)
            + u64::pow(2, Square::B4 as u32)
            + u64::pow(2, Square::E5 as u32)
            + u64::pow(2, Square::F4 as u32)
            + u64::pow(2, Square::G3 as u32);

        assert_eq!(
            attack_tables
                .slider_attack_tables
                .attack_mask(SliderPiece::Bishop, Square::A5)
                .value(),
            desired_a5_attack_mask
        );
        assert_eq!(
            attack_tables
                .slider_attack_tables
                .attack_mask(SliderPiece::Bishop, Square::G7)
                .value(),
            desired_g7_attack_mask
        );
        assert_eq!(
            attack_tables
                .slider_attack_tables
                .attack_mask(SliderPiece::Bishop, Square::D6)
                .value(),
            desired_d6_attack_mask
        );
    }

    #[test]
    fn attack_tables_bishop() {
        let attack_tables = AttackTables::initialise();
        let mut board = Bitboard::new(0);

        let mut desired_attack_table = u64::pow(2, Square::A7 as u32)
            + u64::pow(2, Square::B6 as u32)
            + u64::pow(2, Square::C5 as u32)
            + u64::pow(2, Square::E5 as u32)
            + u64::pow(2, Square::F6 as u32)
            + u64::pow(2, Square::G7 as u32)
            + u64::pow(2, Square::H8 as u32)
            + u64::pow(2, Square::C3 as u32)
            + u64::pow(2, Square::B2 as u32)
            + u64::pow(2, Square::A1 as u32)
            + u64::pow(2, Square::E3 as u32)
            + u64::pow(2, Square::F2 as u32)
            + u64::pow(2, Square::G1 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Bishop, Side::White, Square::D4)
                .value(),
            desired_attack_table
        );

        board.set_bit(Square::C5);

        desired_attack_table -= u64::pow(2, Square::A7 as u32);
        desired_attack_table -= u64::pow(2, Square::B6 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Bishop, Side::White, Square::D4)
                .value(),
            desired_attack_table
        );

        board.set_bit(Square::F2);

        desired_attack_table -= u64::pow(2, Square::G1 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Bishop, Side::White, Square::D4)
                .value(),
            desired_attack_table
        );

        board.set_bit(Square::G7);

        desired_attack_table -= u64::pow(2, Square::H8 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Bishop, Side::White, Square::D4)
                .value(),
            desired_attack_table
        );

        board.pop_bit(Square::G7);
        board.set_bit(Square::H8);

        desired_attack_table += u64::pow(2, Square::H8 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Bishop, Side::White, Square::D4)
                .value(),
            desired_attack_table
        );

        board.set_bit(Square::G8);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Bishop, Side::White, Square::D4)
                .value(),
            desired_attack_table
        );
    }

    #[test]
    fn attack_masks_rook() {
        let attack_tables = AttackTables::initialise();

        let desired_d5_attack_mask = u64::pow(2, Square::D7 as u32)
            + u64::pow(2, Square::D6 as u32)
            + u64::pow(2, Square::B5 as u32)
            + u64::pow(2, Square::C5 as u32)
            + u64::pow(2, Square::E5 as u32)
            + u64::pow(2, Square::F5 as u32)
            + u64::pow(2, Square::G5 as u32)
            + u64::pow(2, Square::D4 as u32)
            + u64::pow(2, Square::D3 as u32)
            + u64::pow(2, Square::D2 as u32);
        let desired_b3_attack_mask = u64::pow(2, Square::B7 as u32)
            + u64::pow(2, Square::B6 as u32)
            + u64::pow(2, Square::B5 as u32)
            + u64::pow(2, Square::B4 as u32)
            + u64::pow(2, Square::B2 as u32)
            + u64::pow(2, Square::C3 as u32)
            + u64::pow(2, Square::D3 as u32)
            + u64::pow(2, Square::E3 as u32)
            + u64::pow(2, Square::F3 as u32)
            + u64::pow(2, Square::G3 as u32);
        let desired_e1_attack_mask = u64::pow(2, Square::E7 as u32)
            + u64::pow(2, Square::E6 as u32)
            + u64::pow(2, Square::E5 as u32)
            + u64::pow(2, Square::E4 as u32)
            + u64::pow(2, Square::E3 as u32)
            + u64::pow(2, Square::E2 as u32)
            + u64::pow(2, Square::B1 as u32)
            + u64::pow(2, Square::C1 as u32)
            + u64::pow(2, Square::D1 as u32)
            + u64::pow(2, Square::F1 as u32)
            + u64::pow(2, Square::G1 as u32);

        assert_eq!(
            attack_tables
                .slider_attack_tables
                .attack_mask(SliderPiece::Rook, Square::D5)
                .value(),
            desired_d5_attack_mask
        );
        assert_eq!(
            attack_tables
                .slider_attack_tables
                .attack_mask(SliderPiece::Rook, Square::B3)
                .value(),
            desired_b3_attack_mask
        );
        assert_eq!(
            attack_tables
                .slider_attack_tables
                .attack_mask(SliderPiece::Rook, Square::E1)
                .value(),
            desired_e1_attack_mask
        );
    }

    #[test]
    fn attack_tables_rook() {
        let attack_tables = AttackTables::initialise();
        let mut board = Bitboard::new(0);

        let mut desired_attack_table = u64::pow(2, Square::E8 as u32)
            + u64::pow(2, Square::E7 as u32)
            + u64::pow(2, Square::E6 as u32)
            + u64::pow(2, Square::E4 as u32)
            + u64::pow(2, Square::E3 as u32)
            + u64::pow(2, Square::E2 as u32)
            + u64::pow(2, Square::E1 as u32)
            + u64::pow(2, Square::A5 as u32)
            + u64::pow(2, Square::B5 as u32)
            + u64::pow(2, Square::C5 as u32)
            + u64::pow(2, Square::D5 as u32)
            + u64::pow(2, Square::F5 as u32)
            + u64::pow(2, Square::G5 as u32)
            + u64::pow(2, Square::H5 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Rook, Side::White, Square::E5)
                .value(),
            desired_attack_table
        );

        board.set_bit(Square::E7);

        desired_attack_table -= u64::pow(2, Square::E8 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Rook, Side::White, Square::E5)
                .value(),
            desired_attack_table
        );

        board.set_bit(Square::E2);

        desired_attack_table -= u64::pow(2, Square::E1 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Rook, Side::White, Square::E5)
                .value(),
            desired_attack_table
        );

        board.set_bit(Square::C5);

        desired_attack_table -= u64::pow(2, Square::A5 as u32);
        desired_attack_table -= u64::pow(2, Square::B5 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Rook, Side::White, Square::E5)
                .value(),
            desired_attack_table
        );

        board.set_bit(Square::G5);

        desired_attack_table -= u64::pow(2, Square::H5 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Rook, Side::White, Square::E5)
                .value(),
            desired_attack_table
        );

        board.pop_bit(Square::G5);
        board.set_bit(Square::H5);

        desired_attack_table += u64::pow(2, Square::H5 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Rook, Side::White, Square::E5)
                .value(),
            desired_attack_table
        );

        board.set_bit(Square::C8);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Rook, Side::White, Square::E5)
                .value(),
            desired_attack_table
        );
    }

    #[test]
    fn attack_tables_queen() {
        let attack_tables = AttackTables::initialise();
        let mut board = Bitboard::new(0);

        let mut desired_attack_table = u64::pow(2, Square::A7 as u32)
            + u64::pow(2, Square::B6 as u32)
            + u64::pow(2, Square::C5 as u32)
            + u64::pow(2, Square::E3 as u32)
            + u64::pow(2, Square::F2 as u32)
            + u64::pow(2, Square::G1 as u32)
            + u64::pow(2, Square::A1 as u32)
            + u64::pow(2, Square::B2 as u32)
            + u64::pow(2, Square::C3 as u32)
            + u64::pow(2, Square::E5 as u32)
            + u64::pow(2, Square::F6 as u32)
            + u64::pow(2, Square::G7 as u32)
            + u64::pow(2, Square::H8 as u32)
            + u64::pow(2, Square::D1 as u32)
            + u64::pow(2, Square::D2 as u32)
            + u64::pow(2, Square::D3 as u32)
            + u64::pow(2, Square::D5 as u32)
            + u64::pow(2, Square::D6 as u32)
            + u64::pow(2, Square::D7 as u32)
            + u64::pow(2, Square::D8 as u32)
            + u64::pow(2, Square::A4 as u32)
            + u64::pow(2, Square::B4 as u32)
            + u64::pow(2, Square::C4 as u32)
            + u64::pow(2, Square::E4 as u32)
            + u64::pow(2, Square::F4 as u32)
            + u64::pow(2, Square::G4 as u32)
            + u64::pow(2, Square::H4 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Queen, Side::White, Square::D4)
                .value(),
            desired_attack_table
        );

        board.set_bit(Square::B6);

        desired_attack_table -= u64::pow(2, Square::A7 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Queen, Side::White, Square::D4)
                .value(),
            desired_attack_table
        );

        board.set_bit(Square::D6);

        desired_attack_table -= u64::pow(2, Square::D8 as u32);
        desired_attack_table -= u64::pow(2, Square::D7 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Queen, Side::White, Square::D4)
                .value(),
            desired_attack_table
        );

        board.set_bit(Square::F6);

        desired_attack_table -= u64::pow(2, Square::G7 as u32);
        desired_attack_table -= u64::pow(2, Square::H8 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Queen, Side::White, Square::D4)
                .value(),
            desired_attack_table
        );

        board.set_bit(Square::B4);

        desired_attack_table -= u64::pow(2, Square::A4 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Queen, Side::White, Square::D4)
                .value(),
            desired_attack_table
        );

        board.set_bit(Square::G4);

        desired_attack_table -= u64::pow(2, Square::H4 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Queen, Side::White, Square::D4)
                .value(),
            desired_attack_table
        );

        board.set_bit(Square::C3);

        desired_attack_table -= u64::pow(2, Square::B2 as u32);
        desired_attack_table -= u64::pow(2, Square::A1 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Queen, Side::White, Square::D4)
                .value(),
            desired_attack_table
        );

        board.set_bit(Square::D3);

        desired_attack_table -= u64::pow(2, Square::D2 as u32);
        desired_attack_table -= u64::pow(2, Square::D1 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Queen, Side::White, Square::D4)
                .value(),
            desired_attack_table
        );

        board.set_bit(Square::E3);

        desired_attack_table -= u64::pow(2, Square::F2 as u32);
        desired_attack_table -= u64::pow(2, Square::G1 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Queen, Side::White, Square::D4)
                .value(),
            desired_attack_table
        );
    }

    #[test]
    fn attack_tables_king() {
        let attack_tables = AttackTables::initialise();

        let desired_b2_attack_table = u64::pow(2, Square::A3 as u32)
            + u64::pow(2, Square::B3 as u32)
            + u64::pow(2, Square::C3 as u32)
            + u64::pow(2, Square::A2 as u32)
            + u64::pow(2, Square::C2 as u32)
            + u64::pow(2, Square::A1 as u32)
            + u64::pow(2, Square::B1 as u32)
            + u64::pow(2, Square::C1 as u32);
        let desired_a1_attack_table = u64::pow(2, Square::A2 as u32)
            + u64::pow(2, Square::B2 as u32)
            + u64::pow(2, Square::B1 as u32);
        let desired_h4_attack_table = u64::pow(2, Square::G5 as u32)
            + u64::pow(2, Square::H5 as u32)
            + u64::pow(2, Square::G4 as u32)
            + u64::pow(2, Square::G3 as u32)
            + u64::pow(2, Square::H3 as u32);

        assert_eq!(
            attack_tables
                .attack_table(Bitboard::new(0), Piece::King, Side::White, Square::B2)
                .value(),
            desired_b2_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(Bitboard::new(0), Piece::King, Side::White, Square::A1)
                .value(),
            desired_a1_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(Bitboard::new(0), Piece::King, Side::White, Square::H4)
                .value(),
            desired_h4_attack_table
        );
    }

    #[test]
    fn generate_magic_numbers() {
        let mut random_state = 1804289383;
        let magic_numbers = MagicNumbers::_initialise(&mut random_state);

        assert_eq!(magic_numbers, MAGIC_NUMBERS)
    }
}
