use super::{Bitboard, Piece, Side, Square};
use num_traits::{FromPrimitive, ToPrimitive};
use once_cell::sync::Lazy;
use strum::IntoEnumIterator;

pub static ATTACK_TABLES: Lazy<AttackTables> = Lazy::new(AttackTables::initialise);

// Magic numbers generated using generate_magic_numbers() with random_state = 1804289383
const MAGIC_NUMBERS: MagicNumbers = MagicNumbers {
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
};

#[derive(Clone, Copy)]
enum LeaperPiece {
    Pawn = 0,
    Knight = 1,
    King = 5,
}

#[derive(Clone, Copy)]
enum SliderPiece {
    Bishop = 2,
    Rook = 3,
}

pub struct AttackTables {
    leaper_attack_tables: LeaperAttackTables,
    slider_attack_tables: SliderAttackTables,
}

impl AttackTables {
    fn initialise() -> Self {
        let leaper_attack_tables = LeaperAttackTables::initialise();
        let slider_attack_tables = SliderAttackTables::initialise();

        Self {
            leaper_attack_tables,
            slider_attack_tables,
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
                Side::White => {
                    self.leaper_attack_tables.white_pawn_attack_tables[square.to_usize().unwrap()]
                }
                Side::Black => {
                    self.leaper_attack_tables.black_pawn_attack_tables[square.to_usize().unwrap()]
                }
            },
            Piece::Knight => {
                self.leaper_attack_tables.knight_attack_tables[square.to_usize().unwrap()]
            }
            Piece::Bishop => {
                let bishop_magic_index =
                    MAGIC_NUMBERS.get_magic_index(board, SliderPiece::Bishop, square);
                self.slider_attack_tables.bishop_attack_tables[square.to_usize().unwrap()]
                    [bishop_magic_index]
            }
            Piece::Rook => {
                let rook_magic_index =
                    MAGIC_NUMBERS.get_magic_index(board, SliderPiece::Rook, square);
                self.slider_attack_tables.rook_attack_tables[square.to_usize().unwrap()]
                    [rook_magic_index]
            }
            Piece::Queen => {
                let bishop_magic_index =
                    MAGIC_NUMBERS.get_magic_index(board, SliderPiece::Bishop, square);
                let rook_magic_index =
                    MAGIC_NUMBERS.get_magic_index(board, SliderPiece::Rook, square);

                self.slider_attack_tables.bishop_attack_tables[square.to_usize().unwrap()]
                    [bishop_magic_index]
                    | self.slider_attack_tables.rook_attack_tables[square.to_usize().unwrap()]
                        [rook_magic_index]
            }
            Piece::King => self.leaper_attack_tables.king_attack_tables[square.to_usize().unwrap()],
        }
    }
}

struct LeaperAttackTables {
    white_pawn_attack_tables: [Bitboard; 64],
    black_pawn_attack_tables: [Bitboard; 64],
    knight_attack_tables: [Bitboard; 64],
    king_attack_tables: [Bitboard; 64],
}

struct SliderAttackTables {
    bishop_attack_masks: [Bitboard; 64],
    rook_attack_masks: [Bitboard; 64],
    bishop_attack_tables: Vec<[Bitboard; 512]>,
    rook_attack_tables: Vec<[Bitboard; 4096]>,
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
        let not_a_file = Bitboard(0xFEFE_FEFE_FEFE_FEFE);
        let not_h_file = Bitboard(0x7F7F_7F7F_7F7F_7F7F);
        let not_ab_file = Bitboard(0xFCFC_FCFC_FCFC_FCFC);
        let not_gh_file = Bitboard(0x3F3F_3F3F_3F3F_3F3F);

        let mut attack_tables = [Bitboard(0); 64];

        for square in Square::iter() {
            let mut bitboard = Bitboard(0);
            let mut attack_table = Bitboard(0);

            bitboard.set_bit(square);

            attack_tables[square.to_usize().unwrap()] = match piece {
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

impl SliderAttackTables {
    fn initialise() -> Self {
        let bishop_attack_masks = Self::generate_attack_masks(SliderPiece::Bishop);
        let rook_attack_masks = Self::generate_attack_masks(SliderPiece::Rook);

        let mut bishop_attack_tables = vec![[Bitboard(0); 512]; 64];
        let mut rook_attack_tables = vec![[Bitboard(0); 4096]; 64];

        for square in Square::iter() {
            let bishop_occupancy_indices =
                1 << bishop_attack_masks[square.to_usize().unwrap()].count_bits();

            for index in 0..bishop_occupancy_indices {
                let occupancy =
                    Self::set_occupancy(index, bishop_attack_masks[square.to_usize().unwrap()]);

                let magic_index = MAGIC_NUMBERS.generate_magic_index(
                    occupancy,
                    bishop_attack_masks[square.to_usize().unwrap()],
                    square,
                    SliderPiece::Bishop,
                );

                bishop_attack_tables[square.to_usize().unwrap()][magic_index] =
                    Self::generate_attack_table(occupancy, SliderPiece::Bishop, square);
            }

            let rook_occupancy_indices =
                1 << rook_attack_masks[square.to_usize().unwrap()].count_bits();

            for index in 0..rook_occupancy_indices {
                let occupancy =
                    Self::set_occupancy(index, rook_attack_masks[square.to_usize().unwrap()]);

                let magic_index = MAGIC_NUMBERS.generate_magic_index(
                    occupancy,
                    rook_attack_masks[square.to_usize().unwrap()],
                    square,
                    SliderPiece::Rook,
                );

                rook_attack_tables[square.to_usize().unwrap()][magic_index] =
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
        let mut attack_masks = [Bitboard(0); 64];

        for square in Square::iter() {
            let mut attack_mask = Bitboard(0);

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

                    attack_masks[square.to_usize().unwrap()] = attack_mask;
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

                    attack_masks[square.to_usize().unwrap()] = attack_mask;
                }
            }
        }

        attack_masks
    }

    fn generate_attack_table(board: Bitboard, piece: SliderPiece, square: Square) -> Bitboard {
        let mut attack_table = Bitboard(0);

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
        let mut occupancy = Bitboard(0);
        let mut count = 0;

        while let Some(square_index) = attack_mask.get_lsb_index() {
            let lsb_square = Square::from_usize(square_index).unwrap();

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
            SliderPiece::Bishop => self.bishop_attack_masks[square.to_usize().unwrap()],
            SliderPiece::Rook => self.rook_attack_masks[square.to_usize().unwrap()],
        }
    }
}

struct MagicNumbers {
    bishop_magic_numbers: [u64; 64],
    rook_magic_numbers: [u64; 64],
}

impl MagicNumbers {
    fn generate_magic_index(
        &self,
        occupancy: Bitboard,
        attack_mask: Bitboard,
        square: Square,
        piece: SliderPiece,
    ) -> usize {
        match piece {
            SliderPiece::Bishop => {
                let magic_index = occupancy
                    .0
                    .overflowing_mul(self.magic_number(piece, square))
                    .0
                    >> (64 - attack_mask.count_bits());

                magic_index as usize
            }
            SliderPiece::Rook => {
                let magic_index = occupancy
                    .0
                    .overflowing_mul(self.magic_number(piece, square))
                    .0
                    >> (64 - attack_mask.count_bits());

                magic_index as usize
            }
        }
    }

    fn magic_number(&self, piece: SliderPiece, square: Square) -> u64 {
        match piece {
            SliderPiece::Bishop => self.bishop_magic_numbers[square.to_usize().unwrap()],
            SliderPiece::Rook => self.rook_magic_numbers[square.to_usize().unwrap()],
        }
    }

    fn get_magic_index(&self, board: Bitboard, piece: SliderPiece, square: Square) -> usize {
        let mut magic_index = board;

        magic_index &= ATTACK_TABLES
            .slider_attack_tables
            .attack_mask(piece, square);
        magic_index = Bitboard(
            magic_index
                .0
                .overflowing_mul(self.magic_number(piece, square))
                .0,
        );
        magic_index >>= 64
            - ATTACK_TABLES
                .slider_attack_tables
                .attack_mask(piece, square)
                .count_bits();

        magic_index.0 as usize
    }

    // Implementation to generate magic numbers taken from
    // https://www.youtube.com/watch?v=UnEu5GOiSEs&list=PLmN0neTso3Jxh8ZIylk74JpwfiWNI76Cs&index=15
    pub fn _new(random_state: &mut u32) -> Self {
        let mut bishop_magic_numbers = [0; 64];
        let mut rook_magic_numbers = [0; 64];

        let slider_attack_tables = SliderAttackTables::initialise();

        for square in Square::iter() {
            rook_magic_numbers[square.to_usize().unwrap()] = Self::_generate_magic_number(
                random_state,
                slider_attack_tables.attack_mask(SliderPiece::Rook, square),
                SliderPiece::Rook,
                square,
            )
        }

        for square in Square::iter() {
            bishop_magic_numbers[square.to_usize().unwrap()] = Self::_generate_magic_number(
                random_state,
                slider_attack_tables.attack_mask(SliderPiece::Bishop, square),
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
    ) -> u64 {
        let mut occupancies = [Bitboard(0); 4096];
        let mut attacks = [Bitboard(0); 4096];

        let occupancy_count = attack_mask.count_bits();
        let occupancy_indices = 1 << occupancy_count;

        for index in 0..occupancy_indices {
            occupancies[index] = SliderAttackTables::set_occupancy(index, attack_mask);
            attacks[index] =
                SliderAttackTables::generate_attack_table(occupancies[index], piece, square);
        }

        'outer: loop {
            let magic_number_candidate = Self::_generate_random_u64_integer(random_state)
                & Self::_generate_random_u64_integer(random_state)
                & Self::_generate_random_u64_integer(random_state);

            let inappropriate_candidate = (attack_mask.0.overflowing_mul(magic_number_candidate).0
                & 0xFF00000000000000)
                .count_ones()
                < 6;

            if inappropriate_candidate {
                continue;
            };

            let mut used_attacks = [Bitboard(0); 4096];

            for i in 0..occupancy_indices {
                let magic_index = ((occupancies[i].0.overflowing_mul(magic_number_candidate).0)
                    >> (64 - occupancy_count)) as usize;

                if used_attacks[magic_index] == 0u64 {
                    used_attacks[magic_index] = attacks[i];
                } else if used_attacks[magic_index] != attacks[i] {
                    continue 'outer;
                }
            }

            return magic_number_candidate;
        }
    }

    fn _generate_random_u64_integer(random_state: &mut u32) -> u64 {
        // `& 0xFFFF` operation cuts off first 16 most significant bits from 32 bit integer
        Self::_mutate_random_state(random_state);
        let random_u64_integer_1 = (*random_state & 0xFFFF) as u64;

        Self::_mutate_random_state(random_state);
        let random_u64_integer_2 = (*random_state & 0xFFFF) as u64;

        Self::_mutate_random_state(random_state);
        let random_u64_integer_3 = (*random_state & 0xFFFF) as u64;

        Self::_mutate_random_state(random_state);
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
        let attack_tables = AttackTables::initialise();

        let desired_h3_attack_table = u64::pow(2, Square::G4 as u32);
        let desired_f5_attack_table =
            u64::pow(2, Square::E6 as u32) + u64::pow(2, Square::G6 as u32);
        let desired_a4_attack_table = u64::pow(2, Square::B5 as u32);

        assert_eq!(
            attack_tables
                .attack_table(Bitboard(0), Piece::Pawn, Side::White, Square::H3)
                .0,
            desired_h3_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(Bitboard(0), Piece::Pawn, Side::White, Square::F5)
                .0,
            desired_f5_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(Bitboard(0), Piece::Pawn, Side::White, Square::A4)
                .0,
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
                .attack_table(Bitboard(0), Piece::Pawn, Side::Black, Square::B4)
                .0,
            desired_b4_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(Bitboard(0), Piece::Pawn, Side::Black, Square::H4)
                .0,
            desired_h4_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(Bitboard(0), Piece::Pawn, Side::Black, Square::A5)
                .0,
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
                .attack_table(Bitboard(0), Piece::Knight, Side::White, Square::G5)
                .0,
            desired_g5_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(Bitboard(0), Piece::Knight, Side::White, Square::E2)
                .0,
            desired_e2_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(Bitboard(0), Piece::Knight, Side::White, Square::F4)
                .0,
            desired_f4_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(Bitboard(0), Piece::Knight, Side::White, Square::B4)
                .0,
            desired_b4_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(Bitboard(0), Piece::Knight, Side::White, Square::A4)
                .0,
            desired_a4_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(Bitboard(0), Piece::Knight, Side::White, Square::H8)
                .0,
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
                .0,
            desired_a5_attack_mask
        );
        assert_eq!(
            attack_tables
                .slider_attack_tables
                .attack_mask(SliderPiece::Bishop, Square::G7)
                .0,
            desired_g7_attack_mask
        );
        assert_eq!(
            attack_tables
                .slider_attack_tables
                .attack_mask(SliderPiece::Bishop, Square::D6)
                .0,
            desired_d6_attack_mask
        );
    }

    #[test]
    fn attack_tables_bishop() {
        let attack_tables = AttackTables::initialise();

        let mut board = Bitboard(0);

        let desired_attack_table = u64::pow(2, Square::A7 as u32)
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
                .0,
            desired_attack_table
        );

        board.set_bit(Square::C5);

        let blocked_desired_attack_table =
            desired_attack_table - u64::pow(2, Square::A7 as u32) - u64::pow(2, Square::B6 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Bishop, Side::White, Square::D4)
                .0,
            blocked_desired_attack_table
        );

        board.set_bit(Square::F2);

        let blocked_desired_attack_table =
            blocked_desired_attack_table - u64::pow(2, Square::G1 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Bishop, Side::White, Square::D4)
                .0,
            blocked_desired_attack_table
        );

        board.set_bit(Square::G7);

        let blocked_desired_attack_table =
            blocked_desired_attack_table - u64::pow(2, Square::H8 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Bishop, Side::White, Square::D4)
                .0,
            blocked_desired_attack_table
        );

        board.pop_bit(Square::G7);
        board.set_bit(Square::H8);

        let blocked_desired_attack_table =
            blocked_desired_attack_table + u64::pow(2, Square::H8 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Bishop, Side::White, Square::D4)
                .0,
            blocked_desired_attack_table
        );

        board.set_bit(Square::G8);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Bishop, Side::White, Square::D4)
                .0,
            blocked_desired_attack_table
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
                .0,
            desired_d5_attack_mask
        );
        assert_eq!(
            attack_tables
                .slider_attack_tables
                .attack_mask(SliderPiece::Rook, Square::B3)
                .0,
            desired_b3_attack_mask
        );
        assert_eq!(
            attack_tables
                .slider_attack_tables
                .attack_mask(SliderPiece::Rook, Square::E1)
                .0,
            desired_e1_attack_mask
        );
    }

    #[test]
    fn attack_tables_rook() {
        let attack_tables = AttackTables::initialise();

        let mut board = Bitboard(0);

        let desired_attack_table = u64::pow(2, Square::E8 as u32)
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
                .0,
            desired_attack_table
        );

        board.set_bit(Square::E7);

        let blocked_desired_attack_table = desired_attack_table - u64::pow(2, Square::E8 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Rook, Side::White, Square::E5)
                .0,
            blocked_desired_attack_table
        );

        board.set_bit(Square::E2);

        let blocked_desired_attack_table =
            blocked_desired_attack_table - u64::pow(2, Square::E1 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Rook, Side::White, Square::E5)
                .0,
            blocked_desired_attack_table
        );

        board.set_bit(Square::C5);

        let blocked_desired_attack_table = blocked_desired_attack_table
            - u64::pow(2, Square::A5 as u32)
            - u64::pow(2, Square::B5 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Rook, Side::White, Square::E5)
                .0,
            blocked_desired_attack_table
        );

        board.set_bit(Square::G5);

        let blocked_desired_attack_table =
            blocked_desired_attack_table - u64::pow(2, Square::H5 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Rook, Side::White, Square::E5)
                .0,
            blocked_desired_attack_table
        );

        board.pop_bit(Square::G5);
        board.set_bit(Square::H5);

        let blocked_desired_attack_table =
            blocked_desired_attack_table + u64::pow(2, Square::H5 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Rook, Side::White, Square::E5)
                .0,
            blocked_desired_attack_table
        );

        board.set_bit(Square::C8);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Rook, Side::White, Square::E5)
                .0,
            blocked_desired_attack_table
        );
    }

    #[test]
    fn attack_tables_queen() {
        let attack_tables = AttackTables::initialise();

        let mut board = Bitboard(0);

        let desired_attack_table = u64::pow(2, Square::A7 as u32)
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
                .0,
            desired_attack_table
        );

        board.set_bit(Square::B6);

        let blocked_desired_attack_table = desired_attack_table - u64::pow(2, Square::A7 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Queen, Side::White, Square::D4)
                .0,
            blocked_desired_attack_table
        );

        board.set_bit(Square::D6);

        let blocked_desired_attack_table = blocked_desired_attack_table
            - u64::pow(2, Square::D8 as u32)
            - u64::pow(2, Square::D7 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Queen, Side::White, Square::D4)
                .0,
            blocked_desired_attack_table
        );

        board.set_bit(Square::F6);

        let blocked_desired_attack_table = blocked_desired_attack_table
            - u64::pow(2, Square::G7 as u32)
            - u64::pow(2, Square::H8 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Queen, Side::White, Square::D4)
                .0,
            blocked_desired_attack_table
        );

        board.set_bit(Square::B4);

        let blocked_desired_attack_table =
            blocked_desired_attack_table - u64::pow(2, Square::A4 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Queen, Side::White, Square::D4)
                .0,
            blocked_desired_attack_table
        );

        board.set_bit(Square::G4);

        let blocked_desired_attack_table =
            blocked_desired_attack_table - u64::pow(2, Square::H4 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Queen, Side::White, Square::D4)
                .0,
            blocked_desired_attack_table
        );

        board.set_bit(Square::C3);

        let blocked_desired_attack_table = blocked_desired_attack_table
            - u64::pow(2, Square::B2 as u32)
            - u64::pow(2, Square::A1 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Queen, Side::White, Square::D4)
                .0,
            blocked_desired_attack_table
        );

        board.set_bit(Square::D3);

        let blocked_desired_attack_table = blocked_desired_attack_table
            - u64::pow(2, Square::D2 as u32)
            - u64::pow(2, Square::D1 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Queen, Side::White, Square::D4)
                .0,
            blocked_desired_attack_table
        );

        board.set_bit(Square::E3);

        let blocked_desired_attack_table = blocked_desired_attack_table
            - u64::pow(2, Square::F2 as u32)
            - u64::pow(2, Square::G1 as u32);

        assert_eq!(
            attack_tables
                .attack_table(board, Piece::Queen, Side::White, Square::D4)
                .0,
            blocked_desired_attack_table
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
                .attack_table(Bitboard(0), Piece::King, Side::White, Square::B2)
                .0,
            desired_b2_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(Bitboard(0), Piece::King, Side::White, Square::A1)
                .0,
            desired_a1_attack_table
        );
        assert_eq!(
            attack_tables
                .attack_table(Bitboard(0), Piece::King, Side::White, Square::H4)
                .0,
            desired_h4_attack_table
        );
    }
}
