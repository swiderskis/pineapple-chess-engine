mod attack_tables;

use self::attack_tables::{AttackTables, LeaperAttackTables, SliderAttackTables};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

pub fn position() {
    let leaper_attack_tables = LeaperAttackTables::initialise();
    let slider_attack_tables = SliderAttackTables::initialise();

    // Magic numbers generated using attack_tables::magic_numbers module,
    // with random_state = 1804289383
    let rook_magic_numbers: [u64; 64] = [
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
    ];

    let bishop_magic_numbers: [u64; 64] = [
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
    ];
}

#[derive(Clone, Copy)]
pub struct Bitboard {
    bitboard: u64,
}

impl Bitboard {
    fn new(bitboard: u64) -> Self {
        Bitboard { bitboard }
    }

    fn get_bit(&self, square: &BoardSquare) -> bool {
        self.bitboard & (1 << square.enumeration()) != 0
    }

    fn set_bit(&mut self, square: &BoardSquare) {
        self.bitboard |= 1 << square.enumeration();
    }

    fn pop_bit(&mut self, square: &BoardSquare) {
        self.bitboard &= !(1 << square.enumeration());
    }

    fn count_bits(&self) -> u32 {
        self.bitboard.count_ones()
    }

    // ls1b = least significant 1st bit
    fn get_ls1b_index(&self) -> Option<u32> {
        if self.is_empty() {
            return None;
        }

        Some(self.bitboard.trailing_zeros())
    }

    fn is_empty(&self) -> bool {
        self.bitboard == 0
    }

    fn print(&self) {
        BoardSquare::iter().for_each(|square| {
            if square.file() == 0 {
                print!("{}   ", (64 - square.enumeration()) / 8);
            }

            print!("{} ", if self.get_bit(&square) { 1 } else { 0 });

            if square.file() == 7 {
                println!("");
            }
        });

        println!("");
        println!("    a b c d e f g h");
        println!("");
        println!("    Bitboard decimal value: {}", self.bitboard);
    }
}

pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

pub enum Side {
    White,
    Black,
    Either,
}

#[derive(Clone, Display, EnumIter, FromPrimitive)]
pub enum BoardSquare {
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
    fn new_from_index(index: u32) -> Self {
        let square_option = Self::from_u32(index);

        match square_option {
            None => panic!("Attempted to convert invalid index into board square"),
            Some(square) => square,
        }
    }

    fn enumeration(&self) -> usize {
        self.clone() as usize
    }

    fn rank(&self) -> usize {
        self.enumeration() / 8
    }

    fn file(&self) -> usize {
        self.enumeration() % 8
    }

    fn to_lowercase_string(&self) -> String {
        self.to_string().to_lowercase()
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

        bitboard1.set_bit(&BoardSquare::H2);
        bitboard2.set_bit(&BoardSquare::G6);
        bitboard3.set_bit(&BoardSquare::B4);

        assert_eq!(
            bitboard1.bitboard,
            u64::pow(2, BoardSquare::H2.enumeration() as u32)
        );
        assert_eq!(
            bitboard2.bitboard,
            u64::pow(2, BoardSquare::G6.enumeration() as u32)
        );
        assert_eq!(
            bitboard3.bitboard,
            u64::pow(2, BoardSquare::B4.enumeration() as u32)
        );
    }

    #[test]
    fn pop_bit() {
        let mut bitboard1 = Bitboard::new(0);
        let mut bitboard2 = Bitboard::new(0);
        let mut bitboard3 = Bitboard::new(0);

        bitboard1.set_bit(&BoardSquare::G5);
        bitboard1.set_bit(&BoardSquare::A8);
        bitboard1.pop_bit(&BoardSquare::G5);

        bitboard2.set_bit(&BoardSquare::C1);
        bitboard2.set_bit(&BoardSquare::A7);
        bitboard2.pop_bit(&BoardSquare::C1);

        bitboard3.set_bit(&BoardSquare::C4);
        bitboard3.set_bit(&BoardSquare::B8);
        bitboard3.pop_bit(&BoardSquare::C4);

        assert_eq!(
            bitboard1.bitboard,
            u64::pow(2, BoardSquare::A8.enumeration() as u32)
        );
        assert_eq!(
            bitboard2.bitboard,
            u64::pow(2, BoardSquare::A7.enumeration() as u32)
        );
        assert_eq!(
            bitboard3.bitboard,
            u64::pow(2, BoardSquare::B8.enumeration() as u32)
        );
    }

    #[test]
    fn pop_unset_bit() {
        let mut bitboard1 = Bitboard::new(0);
        let mut bitboard2 = Bitboard::new(0);

        bitboard1.set_bit(&BoardSquare::F1);
        bitboard1.pop_bit(&BoardSquare::F1);
        bitboard1.pop_bit(&BoardSquare::F1);

        bitboard2.pop_bit(&BoardSquare::G2);

        assert_eq!(bitboard1.bitboard, 0);
        assert_eq!(bitboard2.bitboard, 0);
    }
}
