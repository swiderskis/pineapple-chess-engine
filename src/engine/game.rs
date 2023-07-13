use super::{
    attack_tables::{AttackTablesPub, LeaperAttackTables, SliderAttackTables},
    Bitboard, BoardSquare, EnumToInt, Piece, Side,
};
use num_derive::ToPrimitive;
use strum::IntoEnumIterator;

pub struct Game {
    white_pawns: Bitboard,
    white_knights: Bitboard,
    white_bishops: Bitboard,
    white_rooks: Bitboard,
    white_queens: Bitboard,
    white_king: Bitboard,
    black_pawns: Bitboard,
    black_knights: Bitboard,
    black_bishops: Bitboard,
    black_rooks: Bitboard,
    black_queens: Bitboard,
    black_king: Bitboard,
    side_to_move: Side,
    en_passant_square: Option<BoardSquare>,
    castling_rights: CastlingRights,
}

impl Game {
    pub fn initialise(fen: &str) -> Self {
        let mut white_pawns = Bitboard::new(0);
        let mut white_knights = Bitboard::new(0);
        let mut white_bishops = Bitboard::new(0);
        let mut white_rooks = Bitboard::new(0);
        let mut white_queens = Bitboard::new(0);
        let mut white_king = Bitboard::new(0);

        let mut black_pawns = Bitboard::new(0);
        let mut black_knights = Bitboard::new(0);
        let mut black_bishops = Bitboard::new(0);
        let mut black_rooks = Bitboard::new(0);
        let mut black_queens = Bitboard::new(0);
        let mut black_king = Bitboard::new(0);

        let fen = if fen == "startpos" {
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 "
        } else {
            fen
        };

        let fen: Vec<&str> = fen.split_whitespace().collect();

        let mut square_index = 0;

        fen[0].chars().for_each(|character| match character {
            'P' => {
                white_pawns.set_bit(&BoardSquare::new_from_index(square_index));
                square_index += 1;
            }
            'N' => {
                white_knights.set_bit(&BoardSquare::new_from_index(square_index));
                square_index += 1;
            }
            'B' => {
                white_bishops.set_bit(&BoardSquare::new_from_index(square_index));
                square_index += 1;
            }
            'R' => {
                white_rooks.set_bit(&BoardSquare::new_from_index(square_index));
                square_index += 1;
            }
            'Q' => {
                white_queens.set_bit(&BoardSquare::new_from_index(square_index));
                square_index += 1;
            }
            'K' => {
                white_king.set_bit(&BoardSquare::new_from_index(square_index));
                square_index += 1;
            }
            'p' => {
                black_pawns.set_bit(&BoardSquare::new_from_index(square_index));
                square_index += 1;
            }
            'n' => {
                black_knights.set_bit(&BoardSquare::new_from_index(square_index));
                square_index += 1;
            }
            'b' => {
                black_bishops.set_bit(&BoardSquare::new_from_index(square_index));
                square_index += 1;
            }
            'r' => {
                black_rooks.set_bit(&BoardSquare::new_from_index(square_index));
                square_index += 1;
            }
            'q' => {
                black_queens.set_bit(&BoardSquare::new_from_index(square_index));
                square_index += 1;
            }
            'k' => {
                black_king.set_bit(&BoardSquare::new_from_index(square_index));
                square_index += 1;
            }
            '/' => {}
            '0'..='9' => square_index += character as usize - '0' as usize,
            _ => panic!("Attempted to use invalid character in FEN string"),
        });

        let side_to_move = if fen[1] == "w" {
            Side::White
        } else {
            Side::Black
        };

        let en_passant_square = if fen[3] != "-" {
            Some(BoardSquare::new_from_string(fen[3]))
        } else {
            None
        };

        let castling_rights = CastlingRights::initialise(fen[2]);

        Self {
            white_pawns,
            white_knights,
            white_bishops,
            white_rooks,
            white_queens,
            white_king,
            black_pawns,
            black_knights,
            black_bishops,
            black_rooks,
            black_queens,
            black_king,
            side_to_move,
            en_passant_square,
            castling_rights,
        }
    }

    pub fn print(&self) {
        BoardSquare::iter().for_each(|square| {
            if square.file() == 0 {
                print!("{}   ", (64 - square.as_usize()) / 8);
            }

            match self.piece_at_square(&square) {
                Some((piece, side)) => print!("{} ", Self::get_piece_character(&piece, &side)),
                None => print!(". "),
            }

            if square.file() == 7 {
                println!();
            }
        });

        println!();
        println!("    a b c d e f g h");
        println!();
        println!("Side to move: {:?}", self.side_to_move);
        println!("En passant square: {:?}", self.en_passant_square);
        println!("Castling rights: {}", self.castling_rights.as_string());
    }

    pub fn is_square_attacked(
        &self,
        attacking_side: &Side,
        square: &BoardSquare,
        leaper_attack_tables: &LeaperAttackTables,
        slider_attack_tables: &SliderAttackTables,
    ) -> bool {
        match attacking_side {
            Side::White => {
                let pawn_attacks_square = leaper_attack_tables
                    .attack_table(
                        &self.board(&Side::Either),
                        &Piece::Pawn,
                        &Side::Black,
                        square,
                    )
                    .bitboard
                    & self.piece_bitboard(&Piece::Pawn, attacking_side).bitboard
                    != 0;

                if pawn_attacks_square {
                    return true;
                }
            }
            Side::Black => {
                let pawn_attacks_square = leaper_attack_tables
                    .attack_table(
                        &self.board(&Side::Either),
                        &Piece::Pawn,
                        &Side::White,
                        square,
                    )
                    .bitboard
                    & self.piece_bitboard(&Piece::Pawn, attacking_side).bitboard
                    != 0;

                if pawn_attacks_square {
                    return true;
                }
            }
            Side::Either => {
                panic!("Attempted to check if a square is attacked without specifying a side")
            }
        }

        let knight_attacks_square = leaper_attack_tables
            .attack_table(
                &self.board(&Side::Either),
                &Piece::Knight,
                attacking_side,
                square,
            )
            .bitboard
            & self.piece_bitboard(&Piece::Knight, attacking_side).bitboard
            != 0;

        if knight_attacks_square {
            return true;
        }

        let bishop_attacks_square = slider_attack_tables
            .attack_table(
                &self.board(&Side::Either),
                &Piece::Bishop,
                attacking_side,
                square,
            )
            .bitboard
            & self.piece_bitboard(&Piece::Bishop, attacking_side).bitboard
            != 0;

        if bishop_attacks_square {
            return true;
        }

        let rook_attacks_square = slider_attack_tables
            .attack_table(
                &self.board(&Side::Either),
                &Piece::Rook,
                attacking_side,
                square,
            )
            .bitboard
            & self.piece_bitboard(&Piece::Rook, attacking_side).bitboard
            != 0;

        if rook_attacks_square {
            return true;
        }

        let queen_attacks_square = slider_attack_tables
            .attack_table(
                &self.board(&Side::Either),
                &Piece::Queen,
                attacking_side,
                square,
            )
            .bitboard
            & self.piece_bitboard(&Piece::Queen, attacking_side).bitboard
            != 0;

        if queen_attacks_square {
            return true;
        }

        let king_attacks_square = leaper_attack_tables
            .attack_table(
                &self.board(&Side::Either),
                &Piece::King,
                attacking_side,
                square,
            )
            .bitboard
            & self.piece_bitboard(&Piece::King, attacking_side).bitboard
            != 0;

        if king_attacks_square {
            return true;
        }

        false
    }

    pub fn side_to_move_bitboards(&self) -> [(Bitboard, Piece); 6] {
        match self.side_to_move {
            Side::White => [
                (self.white_pawns, Piece::Pawn),
                (self.white_knights, Piece::Knight),
                (self.white_bishops, Piece::Bishop),
                (self.white_rooks, Piece::Rook),
                (self.white_queens, Piece::Queen),
                (self.white_king, Piece::King),
            ],
            Side::Black => [
                (self.black_pawns, Piece::Pawn),
                (self.black_knights, Piece::Knight),
                (self.black_bishops, Piece::Bishop),
                (self.black_rooks, Piece::Rook),
                (self.black_queens, Piece::Queen),
                (self.black_king, Piece::King),
            ],
            Side::Either => panic!("Attempted to access side bitboards without specifying a side"),
        }
    }

    pub fn board(&self, side: &Side) -> Bitboard {
        match side {
            Side::White => Bitboard::new(
                self.white_pawns.bitboard
                    | self.white_knights.bitboard
                    | self.white_bishops.bitboard
                    | self.white_rooks.bitboard
                    | self.white_queens.bitboard
                    | self.white_king.bitboard,
            ),
            Side::Black => Bitboard::new(
                self.black_pawns.bitboard
                    | self.black_knights.bitboard
                    | self.black_bishops.bitboard
                    | self.black_rooks.bitboard
                    | self.black_queens.bitboard
                    | self.black_king.bitboard,
            ),
            Side::Either => Bitboard::new(
                self.white_pawns.bitboard
                    | self.white_knights.bitboard
                    | self.white_bishops.bitboard
                    | self.white_rooks.bitboard
                    | self.white_queens.bitboard
                    | self.white_king.bitboard
                    | self.black_pawns.bitboard
                    | self.black_knights.bitboard
                    | self.black_bishops.bitboard
                    | self.black_rooks.bitboard
                    | self.black_queens.bitboard
                    | self.black_king.bitboard,
            ),
        }
    }

    pub fn piece_at_square(&self, square: &BoardSquare) -> Option<(Piece, Side)> {
        for bitboard in self.piece_bitboards() {
            if bitboard.0.bit_occupied(square) {
                return Some((bitboard.1, bitboard.2));
            }
        }

        None
    }

    pub fn side_to_move(&self) -> &Side {
        &self.side_to_move
    }

    pub fn en_passant_square(&self) -> &Option<BoardSquare> {
        &self.en_passant_square
    }

    pub fn castling_type_allowed(&self, castling_type: &CastlingType) -> bool {
        self.castling_rights.castling_rights & castling_type.as_u8() != 0
    }

    fn piece_bitboards(&self) -> [(Bitboard, Piece, Side); 12] {
        [
            (self.white_pawns, Piece::Pawn, Side::White),
            (self.white_knights, Piece::Knight, Side::White),
            (self.white_bishops, Piece::Bishop, Side::White),
            (self.white_rooks, Piece::Rook, Side::White),
            (self.white_queens, Piece::Queen, Side::White),
            (self.white_king, Piece::King, Side::White),
            (self.black_pawns, Piece::Pawn, Side::Black),
            (self.black_knights, Piece::Knight, Side::Black),
            (self.black_bishops, Piece::Bishop, Side::Black),
            (self.black_rooks, Piece::Rook, Side::Black),
            (self.black_queens, Piece::Queen, Side::Black),
            (self.black_king, Piece::King, Side::Black),
        ]
    }

    fn piece_bitboard(&self, piece: &Piece, side: &Side) -> Bitboard {
        match side {
            Side::White => match piece {
                Piece::Pawn => self.white_pawns,
                Piece::Knight => self.white_knights,
                Piece::Bishop => self.white_bishops,
                Piece::Rook => self.white_rooks,
                Piece::Queen => self.white_queens,
                Piece::King => self.white_king,
            },
            Side::Black => match piece {
                Piece::Pawn => self.black_pawns,
                Piece::Knight => self.black_knights,
                Piece::Bishop => self.black_bishops,
                Piece::Rook => self.black_rooks,
                Piece::Queen => self.black_queens,
                Piece::King => self.black_king,
            },
            Side::Either => panic!("Attempted to access piece bitboard without specifying a side"),
        }
    }

    fn get_piece_character(piece: &Piece, side: &Side) -> char {
        match side {
            Side::White => match piece {
                Piece::Pawn => 'P',
                Piece::Knight => 'N',
                Piece::Bishop => 'B',
                Piece::Rook => 'R',
                Piece::Queen => 'Q',
                Piece::King => 'K',
            },
            Side::Black => match piece {
                Piece::Pawn => 'p',
                Piece::Knight => 'n',
                Piece::Bishop => 'b',
                Piece::Rook => 'r',
                Piece::Queen => 'q',
                Piece::King => 'k',
            },

            Side::Either => panic!("Attempted to get piece character without specifying side"),
        }
    }
}

struct CastlingRights {
    castling_rights: u8,
}

impl CastlingRights {
    fn initialise(castling_rights_string: &str) -> Self {
        if castling_rights_string == "-" {
            return Self { castling_rights: 0 };
        };

        let mut castling_rights = 0;

        castling_rights_string
            .chars()
            .for_each(|character| match character {
                'K' => castling_rights |= CastlingType::WhiteShort.as_u8(),
                'Q' => castling_rights |= CastlingType::WhiteLong.as_u8(),
                'k' => castling_rights |= CastlingType::BlackShort.as_u8(),
                'q' => castling_rights |= CastlingType::BlackLong.as_u8(),
                _ => panic!("Invalid character used when attempting to initialise castling rights"),
            });

        Self { castling_rights }
    }

    fn as_string(&self) -> String {
        let mut castling_rights_string = String::new();

        if self.castling_rights & CastlingType::WhiteShort.as_u8() != 0 {
            castling_rights_string.push('K');
        }

        if self.castling_rights & CastlingType::WhiteLong.as_u8() != 0 {
            castling_rights_string.push('Q');
        }

        if self.castling_rights & CastlingType::BlackShort.as_u8() != 0 {
            castling_rights_string.push('k');
        }

        if self.castling_rights & CastlingType::BlackLong.as_u8() != 0 {
            castling_rights_string.push('q');
        }

        castling_rights_string
    }
}

#[derive(ToPrimitive)]
pub enum CastlingType {
    WhiteShort = 0b1000,
    WhiteLong = 0b0100,
    BlackShort = 0b0010,
    BlackLong = 0b0001,
}

impl EnumToInt for CastlingType {}

impl CastlingType {
    pub fn move_string(&self) -> &str {
        match self {
            Self::WhiteShort => "e1g1",
            Self::WhiteLong => "e1c1",
            Self::BlackShort => "e8g8",
            Self::BlackLong => "e8c8",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tricky_position() {
        let game = Game::initialise(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1 ",
        );

        let desired_white_pawns_bitboard = u64::pow(2, BoardSquare::A2 as u32)
            + u64::pow(2, BoardSquare::B2 as u32)
            + u64::pow(2, BoardSquare::C2 as u32)
            + u64::pow(2, BoardSquare::D5 as u32)
            + u64::pow(2, BoardSquare::E4 as u32)
            + u64::pow(2, BoardSquare::F2 as u32)
            + u64::pow(2, BoardSquare::G2 as u32)
            + u64::pow(2, BoardSquare::H2 as u32);
        let desired_white_knights_bitboard =
            u64::pow(2, BoardSquare::C3 as u32) + u64::pow(2, BoardSquare::E5 as u32);
        let desired_white_bishops_bitboard =
            u64::pow(2, BoardSquare::D2 as u32) + u64::pow(2, BoardSquare::E2 as u32);
        let desired_white_rooks_bitboard =
            u64::pow(2, BoardSquare::A1 as u32) + u64::pow(2, BoardSquare::H1 as u32);
        let desired_white_queens_bitboard = u64::pow(2, BoardSquare::F3 as u32);
        let desired_white_king_bitboard = u64::pow(2, BoardSquare::E1 as u32);

        let desired_black_pawns_bitboard = u64::pow(2, BoardSquare::A7 as u32)
            + u64::pow(2, BoardSquare::B4 as u32)
            + u64::pow(2, BoardSquare::C7 as u32)
            + u64::pow(2, BoardSquare::D7 as u32)
            + u64::pow(2, BoardSquare::E6 as u32)
            + u64::pow(2, BoardSquare::F7 as u32)
            + u64::pow(2, BoardSquare::G6 as u32)
            + u64::pow(2, BoardSquare::H3 as u32);
        let desired_black_knights_bitboard =
            u64::pow(2, BoardSquare::B6 as u32) + u64::pow(2, BoardSquare::F6 as u32);
        let desired_black_bishops_bitboard =
            u64::pow(2, BoardSquare::A6 as u32) + u64::pow(2, BoardSquare::G7 as u32);
        let desired_black_rooks_bitboard =
            u64::pow(2, BoardSquare::A8 as u32) + u64::pow(2, BoardSquare::H8 as u32);
        let desired_black_queens_bitboard = u64::pow(2, BoardSquare::E7 as u32);
        let desired_black_king_bitboard = u64::pow(2, BoardSquare::E8 as u32);

        assert_eq!(game.white_pawns.bitboard, desired_white_pawns_bitboard);
        assert_eq!(game.white_knights.bitboard, desired_white_knights_bitboard);
        assert_eq!(game.white_bishops.bitboard, desired_white_bishops_bitboard);
        assert_eq!(game.white_rooks.bitboard, desired_white_rooks_bitboard);
        assert_eq!(game.white_queens.bitboard, desired_white_queens_bitboard);
        assert_eq!(game.white_king.bitboard, desired_white_king_bitboard);
        assert_eq!(game.black_pawns.bitboard, desired_black_pawns_bitboard);
        assert_eq!(game.black_knights.bitboard, desired_black_knights_bitboard);
        assert_eq!(game.black_bishops.bitboard, desired_black_bishops_bitboard);
        assert_eq!(game.black_rooks.bitboard, desired_black_rooks_bitboard);
        assert_eq!(game.black_queens.bitboard, desired_black_queens_bitboard);
        assert_eq!(game.black_king.bitboard, desired_black_king_bitboard);
    }

    #[test]
    fn killer_position() {
        let game = Game::initialise(
            "rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/3P3P/P1P1P3/RNBQKBNR w KQkq e6 0 1 ",
        );

        let desired_white_pawns_bitboard = u64::pow(2, BoardSquare::A2 as u32)
            + u64::pow(2, BoardSquare::B4 as u32)
            + u64::pow(2, BoardSquare::C2 as u32)
            + u64::pow(2, BoardSquare::D3 as u32)
            + u64::pow(2, BoardSquare::D4 as u32)
            + u64::pow(2, BoardSquare::E2 as u32)
            + u64::pow(2, BoardSquare::F5 as u32)
            + u64::pow(2, BoardSquare::G7 as u32)
            + u64::pow(2, BoardSquare::H3 as u32);
        let desired_white_knights_bitboard =
            u64::pow(2, BoardSquare::B1 as u32) + u64::pow(2, BoardSquare::G1 as u32);
        let desired_white_bishops_bitboard =
            u64::pow(2, BoardSquare::C1 as u32) + u64::pow(2, BoardSquare::F1 as u32);
        let desired_white_rooks_bitboard =
            u64::pow(2, BoardSquare::A1 as u32) + u64::pow(2, BoardSquare::H1 as u32);
        let desired_white_queens_bitboard = u64::pow(2, BoardSquare::D1 as u32);
        let desired_white_king_bitboard = u64::pow(2, BoardSquare::E1 as u32);

        let desired_black_pawns_bitboard = u64::pow(2, BoardSquare::A7 as u32)
            + u64::pow(2, BoardSquare::B7 as u32)
            + u64::pow(2, BoardSquare::C5 as u32)
            + u64::pow(2, BoardSquare::D7 as u32)
            + u64::pow(2, BoardSquare::E5 as u32)
            + u64::pow(2, BoardSquare::F7 as u32)
            + u64::pow(2, BoardSquare::H7 as u32);
        let desired_black_knights_bitboard = u64::pow(2, BoardSquare::B8 as u32);
        let desired_black_bishops_bitboard =
            u64::pow(2, BoardSquare::C8 as u32) + u64::pow(2, BoardSquare::F8 as u32);
        let desired_black_rooks_bitboard =
            u64::pow(2, BoardSquare::A8 as u32) + u64::pow(2, BoardSquare::H8 as u32);
        let desired_black_queens_bitboard = u64::pow(2, BoardSquare::D8 as u32);
        let desired_black_king_bitboard = u64::pow(2, BoardSquare::E8 as u32);

        assert_eq!(game.white_pawns.bitboard, desired_white_pawns_bitboard);
        assert_eq!(game.white_knights.bitboard, desired_white_knights_bitboard);
        assert_eq!(game.white_bishops.bitboard, desired_white_bishops_bitboard);
        assert_eq!(game.white_rooks.bitboard, desired_white_rooks_bitboard);
        assert_eq!(game.white_queens.bitboard, desired_white_queens_bitboard);
        assert_eq!(game.white_king.bitboard, desired_white_king_bitboard);
        assert_eq!(game.black_pawns.bitboard, desired_black_pawns_bitboard);
        assert_eq!(game.black_knights.bitboard, desired_black_knights_bitboard);
        assert_eq!(game.black_bishops.bitboard, desired_black_bishops_bitboard);
        assert_eq!(game.black_rooks.bitboard, desired_black_rooks_bitboard);
        assert_eq!(game.black_queens.bitboard, desired_black_queens_bitboard);
        assert_eq!(game.black_king.bitboard, desired_black_king_bitboard);
    }

    #[test]
    fn cmk_position() {
        let game = Game::initialise(
            "r2q1rk1/ppp2ppp/2n1bn2/2b1p3/3pP3/3P1NPP/PPP1NPB1/R1BQ1RK1 b - - 0 9",
        );

        let desired_white_pawns_bitboard = u64::pow(2, BoardSquare::A2 as u32)
            + u64::pow(2, BoardSquare::B2 as u32)
            + u64::pow(2, BoardSquare::C2 as u32)
            + u64::pow(2, BoardSquare::D3 as u32)
            + u64::pow(2, BoardSquare::E4 as u32)
            + u64::pow(2, BoardSquare::F2 as u32)
            + u64::pow(2, BoardSquare::G3 as u32)
            + u64::pow(2, BoardSquare::H3 as u32);
        let desired_white_knights_bitboard =
            u64::pow(2, BoardSquare::E2 as u32) + u64::pow(2, BoardSquare::F3 as u32);
        let desired_white_bishops_bitboard =
            u64::pow(2, BoardSquare::C1 as u32) + u64::pow(2, BoardSquare::G2 as u32);
        let desired_white_rooks_bitboard =
            u64::pow(2, BoardSquare::A1 as u32) + u64::pow(2, BoardSquare::F1 as u32);
        let desired_white_queens_bitboard = u64::pow(2, BoardSquare::D1 as u32);
        let desired_white_king_bitboard = u64::pow(2, BoardSquare::G1 as u32);

        let desired_black_pawns_bitboard = u64::pow(2, BoardSquare::A7 as u32)
            + u64::pow(2, BoardSquare::B7 as u32)
            + u64::pow(2, BoardSquare::C7 as u32)
            + u64::pow(2, BoardSquare::D4 as u32)
            + u64::pow(2, BoardSquare::E5 as u32)
            + u64::pow(2, BoardSquare::F7 as u32)
            + u64::pow(2, BoardSquare::G7 as u32)
            + u64::pow(2, BoardSquare::H7 as u32);
        let desired_black_knights_bitboard =
            u64::pow(2, BoardSquare::C6 as u32) + u64::pow(2, BoardSquare::F6 as u32);
        let desired_black_bishops_bitboard =
            u64::pow(2, BoardSquare::C5 as u32) + u64::pow(2, BoardSquare::E6 as u32);
        let desired_black_rooks_bitboard =
            u64::pow(2, BoardSquare::A8 as u32) + u64::pow(2, BoardSquare::F8 as u32);
        let desired_black_queens_bitboard = u64::pow(2, BoardSquare::D8 as u32);
        let desired_black_king_bitboard = u64::pow(2, BoardSquare::G8 as u32);

        assert_eq!(game.white_pawns.bitboard, desired_white_pawns_bitboard);
        assert_eq!(game.white_knights.bitboard, desired_white_knights_bitboard);
        assert_eq!(game.white_bishops.bitboard, desired_white_bishops_bitboard);
        assert_eq!(game.white_rooks.bitboard, desired_white_rooks_bitboard);
        assert_eq!(game.white_queens.bitboard, desired_white_queens_bitboard);
        assert_eq!(game.white_king.bitboard, desired_white_king_bitboard);
        assert_eq!(game.black_pawns.bitboard, desired_black_pawns_bitboard);
        assert_eq!(game.black_knights.bitboard, desired_black_knights_bitboard);
        assert_eq!(game.black_bishops.bitboard, desired_black_bishops_bitboard);
        assert_eq!(game.black_rooks.bitboard, desired_black_rooks_bitboard);
        assert_eq!(game.black_queens.bitboard, desired_black_queens_bitboard);
        assert_eq!(game.black_king.bitboard, desired_black_king_bitboard);
    }
}
