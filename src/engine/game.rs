use super::{Bitboard, BoardSquare, EnumToInt, Piece, Side};
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
    pub fn from_fen(fen: String) -> Self {
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

        if fen == "startpos" {
            return Game::start_position();
        }

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
            side_to_move: if fen[1] == "w" {
                Side::White
            } else {
                Side::Black
            },
            en_passant_square: if fen[3] == "-" {
                None
            } else {
                Some(BoardSquare::new_from_string(fen[3]))
            },
            castling_rights: CastlingRights::initialise(fen[2]),
        }
    }

    fn start_position() -> Self {
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

        white_pawns.set_bit(&BoardSquare::A2);
        white_pawns.set_bit(&BoardSquare::B2);
        white_pawns.set_bit(&BoardSquare::C2);
        white_pawns.set_bit(&BoardSquare::D2);
        white_pawns.set_bit(&BoardSquare::E2);
        white_pawns.set_bit(&BoardSquare::F2);
        white_pawns.set_bit(&BoardSquare::G2);
        white_pawns.set_bit(&BoardSquare::H2);

        white_knights.set_bit(&BoardSquare::B1);
        white_knights.set_bit(&BoardSquare::G1);

        white_bishops.set_bit(&BoardSquare::C1);
        white_bishops.set_bit(&BoardSquare::F1);

        white_rooks.set_bit(&BoardSquare::A1);
        white_rooks.set_bit(&BoardSquare::H1);

        white_queens.set_bit(&BoardSquare::D1);

        white_king.set_bit(&BoardSquare::E1);

        black_pawns.set_bit(&BoardSquare::A7);
        black_pawns.set_bit(&BoardSquare::B7);
        black_pawns.set_bit(&BoardSquare::C7);
        black_pawns.set_bit(&BoardSquare::D7);
        black_pawns.set_bit(&BoardSquare::E7);
        black_pawns.set_bit(&BoardSquare::F7);
        black_pawns.set_bit(&BoardSquare::G7);
        black_pawns.set_bit(&BoardSquare::H7);

        black_knights.set_bit(&BoardSquare::B8);
        black_knights.set_bit(&BoardSquare::G8);

        black_bishops.set_bit(&BoardSquare::C8);
        black_bishops.set_bit(&BoardSquare::F8);

        black_rooks.set_bit(&BoardSquare::A8);
        black_rooks.set_bit(&BoardSquare::H8);

        black_queens.set_bit(&BoardSquare::D8);

        black_king.set_bit(&BoardSquare::E8);

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
            side_to_move: Side::White,
            en_passant_square: None,
            castling_rights: CastlingRights::initialise("KQkq"),
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

    fn piece_at_square(&self, square: &BoardSquare) -> Option<(Piece, Side)> {
        for bitboard in self.piece_bitboards() {
            if bitboard.0.bit_occupied(square) {
                return Some((bitboard.1, bitboard.2));
            }
        }

        None
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
                'K' => castling_rights |= CastlingTypes::WhiteShort.as_u8(),
                'Q' => castling_rights |= CastlingTypes::WhiteLong.as_u8(),
                'k' => castling_rights |= CastlingTypes::BlackShort.as_u8(),
                'q' => castling_rights |= CastlingTypes::BlackLong.as_u8(),
                _ => panic!("Invalid character used when attempting to initialise castling rights"),
            });

        Self { castling_rights }
    }

    fn as_string(&self) -> String {
        let mut castling_rights_string = String::new();

        if self.castling_rights & CastlingTypes::WhiteShort.as_u8() != 0 {
            castling_rights_string.push('K');
        }

        if self.castling_rights & CastlingTypes::WhiteLong.as_u8() != 0 {
            castling_rights_string.push('Q');
        }

        if self.castling_rights & CastlingTypes::BlackShort.as_u8() != 0 {
            castling_rights_string.push('k');
        }

        if self.castling_rights & CastlingTypes::BlackLong.as_u8() != 0 {
            castling_rights_string.push('q');
        }

        castling_rights_string
    }
}

#[derive(ToPrimitive)]
enum CastlingTypes {
    WhiteShort = 0b1000,
    WhiteLong = 0b0100,
    BlackShort = 0b0010,
    BlackLong = 0b0001,
}

impl EnumToInt for CastlingTypes {}
