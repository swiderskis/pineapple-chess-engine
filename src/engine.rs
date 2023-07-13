mod attack_tables;
mod game;

use self::{
    attack_tables::{AttackTablesPub, LeaperAttackTables, SliderAttackTables},
    game::{CastlingType, Game},
};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use std::str::FromStr;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

pub fn position() {
    let game = Game::initialise("startpos");

    game.print();

    generate_moves(&game);
}

fn generate_moves(game: &Game) {
    let leaper_attack_tables = LeaperAttackTables::initialise();
    let slider_attack_tables = SliderAttackTables::initialise();

    game.side_to_move_bitboards()
        .iter()
        .for_each(|bitboard_info| {
            let (mut bitboard, piece) = bitboard_info;

            while let Some(source_square_index) = bitboard.get_ls1b_index() {
                match piece {
                    Piece::Pawn => {
                        generate_pawn_moves(source_square_index, game, &leaper_attack_tables);
                    }
                    Piece::Knight => {}
                    Piece::Bishop => {}
                    Piece::Rook => {}
                    Piece::Queen => {}
                    Piece::King => {
                        generate_castling_moves(game, &leaper_attack_tables, &slider_attack_tables);
                    }
                }

                bitboard.pop_bit(&BoardSquare::new_from_index(source_square_index));
            }
        });
}

fn generate_pawn_moves(
    source_square_index: usize,
    game: &Game,
    leaper_attack_tables: &LeaperAttackTables,
) {
    // Bitboards with 2nd and 7th ranks initialised to 1
    let second_rank = Bitboard::new(71776119061217280);
    let seventh_rank = Bitboard::new(65280);

    let side = game.side_to_move();

    let source_square = BoardSquare::new_from_index(source_square_index);
    let target_square = if matches!(side, Side::White) {
        BoardSquare::new_from_index(source_square_index - 8)
    } else {
        BoardSquare::new_from_index(source_square_index + 8)
    };

    let single_piece = Bitboard::from_square(&source_square);

    let piece_on_second_rank = second_rank.bitboard & single_piece.bitboard != 0;
    let piece_on_seventh_rank = seventh_rank.bitboard & single_piece.bitboard != 0;

    let source_square_string = source_square.to_lowercase_string();
    let target_square_string = target_square.to_lowercase_string();

    if ((matches!(side, Side::White) && piece_on_seventh_rank)
        || (matches!(side, Side::Black) && piece_on_second_rank))
        && game.piece_at_square(&target_square).is_none()
    {
        println!("{}{}q", source_square_string, target_square_string);
        println!("{}{}r", source_square_string, target_square_string);
        println!("{}{}b", source_square_string, target_square_string);
        println!("{}{}n", source_square_string, target_square_string);
    } else if game.piece_at_square(&target_square).is_none() {
        println!("{}{}", source_square_string, target_square_string);
    }

    let double_push_target_square = if matches!(side, Side::White) && piece_on_second_rank {
        Some(BoardSquare::new_from_index(source_square_index - 16))
    } else if matches!(side, Side::Black) && piece_on_seventh_rank {
        Some(BoardSquare::new_from_index(source_square_index + 16))
    } else {
        None
    };

    let single_push_target_square = target_square;

    if let Some(target_square) = double_push_target_square {
        if game.piece_at_square(&single_push_target_square).is_none() {
            let target_square_empty = game.piece_at_square(&target_square).is_none();

            let target_square_string = target_square.to_lowercase_string();

            if target_square_empty {
                println!("{}{}", source_square_string, target_square_string);
            }
        }
    }

    let mut attacks = Bitboard::new(
        leaper_attack_tables
            .attack_table(
                &game.board(&Side::Either),
                &Piece::Pawn,
                side,
                &source_square,
            )
            .bitboard
            & game.board(&side.opponent_side()).bitboard,
    );

    while let Some(target_square_index) = attacks.get_ls1b_index() {
        let target_square = BoardSquare::new_from_index(target_square_index);

        let target_square_string = target_square.to_lowercase_string();

        if (matches!(side, Side::White) && piece_on_seventh_rank)
            || (matches!(side, Side::Black) && piece_on_second_rank)
        {
            println!("{}{}q", source_square_string, target_square_string);
            println!("{}{}r", source_square_string, target_square_string);
            println!("{}{}b", source_square_string, target_square_string);
            println!("{}{}n", source_square_string, target_square_string);
        } else {
            println!("{}{}", source_square_string, target_square_string);
        }

        attacks.pop_bit(&target_square);
    }

    if let Some(target_square) = game.en_passant_square() {
        let target_square_string = target_square.to_lowercase_string();

        let en_passant_square_attacked = leaper_attack_tables
            .attack_table(
                &game.board(&Side::Either),
                &Piece::Pawn,
                side,
                &source_square,
            )
            .bitboard
            & Bitboard::from_square(target_square).bitboard
            != 0;

        if en_passant_square_attacked {
            println!("{}{}", source_square_string, target_square_string);
        }
    }
}

fn generate_castling_moves(
    game: &Game,
    leaper_attack_tables: &LeaperAttackTables,
    slider_attack_tables: &SliderAttackTables,
) {
    let side = game.side_to_move();

    let (b_file_square, c_file_square, d_file_square, e_file_square, f_file_square, g_file_square) =
        if matches!(side, Side::White) {
            (
                BoardSquare::B1,
                BoardSquare::C1,
                BoardSquare::D1,
                BoardSquare::E1,
                BoardSquare::F1,
                BoardSquare::G1,
            )
        } else {
            (
                BoardSquare::B8,
                BoardSquare::C8,
                BoardSquare::D8,
                BoardSquare::E8,
                BoardSquare::F8,
                BoardSquare::G8,
            )
        };
    let (short_castle, long_castle) = if matches!(side, Side::White) {
        (CastlingType::WhiteShort, CastlingType::WhiteLong)
    } else {
        (CastlingType::BlackShort, CastlingType::BlackLong)
    };

    let d_file_square_attacked = game.is_square_attacked(
        &side.opponent_side(),
        &d_file_square,
        leaper_attack_tables,
        slider_attack_tables,
    );
    let e_file_square_attacked = game.is_square_attacked(
        &side.opponent_side(),
        &e_file_square,
        leaper_attack_tables,
        slider_attack_tables,
    );
    let f_file_square_attacked = game.is_square_attacked(
        &side.opponent_side(),
        &f_file_square,
        leaper_attack_tables,
        slider_attack_tables,
    );

    if game.piece_at_square(&f_file_square).is_none()
        && game.piece_at_square(&g_file_square).is_none()
        && !e_file_square_attacked
        && !f_file_square_attacked
        && game.castling_type_allowed(&short_castle)
    {
        println!("{}", short_castle.move_string());
    }

    if game.piece_at_square(&b_file_square).is_none()
        && game.piece_at_square(&c_file_square).is_none()
        && game.piece_at_square(&d_file_square).is_none()
        && !d_file_square_attacked
        && !e_file_square_attacked
        && game.castling_type_allowed(&long_castle)
    {
        println!("{}", long_castle.move_string());
    }
}

#[derive(Clone, Copy)]
pub struct Bitboard {
    bitboard: u64,
}

impl Bitboard {
    fn new(bitboard: u64) -> Self {
        Bitboard { bitboard }
    }

    fn from_square(square: &BoardSquare) -> Self {
        let mut bitboard = Bitboard::new(0);

        bitboard.set_bit(square);

        bitboard
    }

    fn bit_occupied(&self, square: &BoardSquare) -> bool {
        self.bitboard & (1 << square.as_usize()) != 0
    }

    fn set_bit(&mut self, square: &BoardSquare) {
        self.bitboard |= 1 << square.as_usize();
    }

    fn pop_bit(&mut self, square: &BoardSquare) {
        self.bitboard &= !(1 << square.as_usize());
    }

    fn count_bits(&self) -> u32 {
        self.bitboard.count_ones()
    }

    // ls1b = least significant 1st bit
    fn get_ls1b_index(&self) -> Option<usize> {
        if self.is_empty() {
            return None;
        }

        Some(self.bitboard.trailing_zeros() as usize)
    }

    fn is_empty(&self) -> bool {
        self.bitboard == 0
    }

    fn print(&self) {
        BoardSquare::iter().for_each(|square| {
            if square.file() == 0 {
                print!("{}   ", (64 - square.as_usize() / 8));
            }

            print!("{} ", if self.bit_occupied(&square) { 1 } else { 0 });

            if square.file() == 7 {
                println!();
            }
        });

        println!();
        println!("    a b c d e f g h");
        println!();
        println!("    Bitboard decimal value: {}", self.bitboard);
    }
}

trait EnumToInt: ToPrimitive {
    fn as_usize(&self) -> usize {
        match self.to_usize() {
            Some(value) => value,
            None => panic!("Failed to convert enum to usize type"),
        }
    }

    fn as_u8(&self) -> u8 {
        match self.to_u8() {
            Some(value) => value,
            None => panic!("Failed to convert enum to u8 type"),
        }
    }
}

#[derive(Debug)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug)]
pub enum Side {
    White,
    Black,
    Either,
}

impl Side {
    pub fn opponent_side(&self) -> Side {
        match self {
            Self::White => Side::Black,
            Self::Black => Side::White,
            Self::Either => panic!("Attempted to get the opposing side without specifying a side"),
        }
    }
}

#[derive(Debug, Display, EnumIter, EnumString, FromPrimitive, ToPrimitive)]
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

impl EnumToInt for BoardSquare {}

impl BoardSquare {
    fn new_from_index(index: usize) -> Self {
        let square_option = Self::from_usize(index);

        match square_option {
            Some(square) => square,
            None => panic!("Attempted to convert invalid index into board square"),
        }
    }

    fn new_from_string(square: &str) -> Self {
        match BoardSquare::from_str(&square.to_uppercase()) {
            Ok(square) => square,
            Err(_) => panic!("Attempted to convert invalid string slice into board square"),
        }
    }

    fn rank(&self) -> usize {
        self.as_usize() / 8
    }

    fn file(&self) -> usize {
        self.as_usize() % 8
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
            u64::pow(2, BoardSquare::H2.as_usize() as u32)
        );
        assert_eq!(
            bitboard2.bitboard,
            u64::pow(2, BoardSquare::G6.as_usize() as u32)
        );
        assert_eq!(
            bitboard3.bitboard,
            u64::pow(2, BoardSquare::B4.as_usize() as u32)
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
            u64::pow(2, BoardSquare::A8.as_usize() as u32)
        );
        assert_eq!(
            bitboard2.bitboard,
            u64::pow(2, BoardSquare::A7.as_usize() as u32)
        );
        assert_eq!(
            bitboard3.bitboard,
            u64::pow(2, BoardSquare::B8.as_usize() as u32)
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
