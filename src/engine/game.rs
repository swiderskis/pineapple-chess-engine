use super::{
    attack_tables,
    moves::{Move, MoveFlag, MoveList, MoveType},
};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{AsPrimitive, FromPrimitive, ToPrimitive, Unsigned};
use std::{
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not, Shl, Shr, ShrAssign},
    str::FromStr,
    time::Instant,
};
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

#[derive(Clone)]
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
    en_passant_square: Option<Square>,
    castling_rights: CastlingRights,
}

impl Game {
    pub fn initialise(fen: &str) -> Self {
        let mut white_pawns = Bitboard(0);
        let mut white_knights = Bitboard(0);
        let mut white_bishops = Bitboard(0);
        let mut white_rooks = Bitboard(0);
        let mut white_queens = Bitboard(0);
        let mut white_king = Bitboard(0);

        let mut black_pawns = Bitboard(0);
        let mut black_knights = Bitboard(0);
        let mut black_bishops = Bitboard(0);
        let mut black_rooks = Bitboard(0);
        let mut black_queens = Bitboard(0);
        let mut black_king = Bitboard(0);

        let fen = if fen == "startpos" {
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        } else {
            fen
        };

        let fen: Vec<&str> = fen.split_whitespace().collect();

        let mut square_index = 0;

        for character in fen[0].chars() {
            match character {
                'P' => {
                    white_pawns.set_bit(Square::from_usize(square_index).unwrap());
                    square_index += 1;
                }
                'N' => {
                    white_knights.set_bit(Square::from_usize(square_index).unwrap());
                    square_index += 1;
                }
                'B' => {
                    white_bishops.set_bit(Square::from_usize(square_index).unwrap());
                    square_index += 1;
                }
                'R' => {
                    white_rooks.set_bit(Square::from_usize(square_index).unwrap());
                    square_index += 1;
                }
                'Q' => {
                    white_queens.set_bit(Square::from_usize(square_index).unwrap());
                    square_index += 1;
                }
                'K' => {
                    white_king.set_bit(Square::from_usize(square_index).unwrap());
                    square_index += 1;
                }
                'p' => {
                    black_pawns.set_bit(Square::from_usize(square_index).unwrap());
                    square_index += 1;
                }
                'n' => {
                    black_knights.set_bit(Square::from_usize(square_index).unwrap());
                    square_index += 1;
                }
                'b' => {
                    black_bishops.set_bit(Square::from_usize(square_index).unwrap());
                    square_index += 1;
                }
                'r' => {
                    black_rooks.set_bit(Square::from_usize(square_index).unwrap());
                    square_index += 1;
                }
                'q' => {
                    black_queens.set_bit(Square::from_usize(square_index).unwrap());
                    square_index += 1;
                }
                'k' => {
                    black_king.set_bit(Square::from_usize(square_index).unwrap());
                    square_index += 1;
                }
                '0'..='9' => square_index += character as usize - '0' as usize,
                _ => {}
            }
        }

        let side_to_move = if fen[1] == "w" {
            Side::White
        } else {
            Side::Black
        };

        let en_passant_square = if fen[3] != "-" {
            Some(Square::from_str(fen[3].to_uppercase().as_str()).unwrap())
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

    pub fn make_move(&mut self, mv: Move, move_flag: MoveFlag) -> Result<(), ()> {
        if move_flag == MoveFlag::Capture && mv.move_type() != MoveType::Capture {
            return Err(());
        }

        let mut game_clone = self.clone();

        let side = game_clone.side_to_move;
        let opponent_side = game_clone.side_to_move.opponent_side();

        game_clone
            .mut_piece_bitboard(mv.piece(), side)
            .pop_bit(mv.source_square());

        if let Some(promoted_piece) = mv.promoted_piece() {
            game_clone
                .mut_piece_bitboard(promoted_piece, side)
                .set_bit(mv.target_square());
        } else {
            game_clone
                .mut_piece_bitboard(mv.piece(), side)
                .set_bit(mv.target_square());
        }

        let target_square_index = mv.target_square().to_usize().unwrap();

        match mv.move_type() {
            MoveType::Capture => {
                let (a_file_square, h_file_square) = match opponent_side {
                    Side::White => (Square::A1, Square::H1),
                    Side::Black => (Square::A8, Square::H8),
                };
                let (short_castle, long_castle) = match opponent_side {
                    Side::White => (CastlingType::WhiteShort, CastlingType::WhiteLong),
                    Side::Black => (CastlingType::BlackShort, CastlingType::BlackLong),
                };

                if mv.target_square() == a_file_square {
                    game_clone.castling_rights.remove_castling_type(long_castle)
                } else if mv.target_square() == h_file_square {
                    game_clone
                        .castling_rights
                        .remove_castling_type(short_castle)
                }

                for piece in Piece::iter() {
                    game_clone
                        .mut_piece_bitboard(piece, opponent_side)
                        .pop_bit(mv.target_square());
                }
            }
            MoveType::DoublePawnPush => {
                let en_passant_square = match side {
                    Side::White => Square::from_usize(target_square_index + 8),
                    Side::Black => Square::from_usize(target_square_index - 8),
                }
                .unwrap();

                game_clone.en_passant_square = Some(en_passant_square);
            }
            MoveType::EnPassant => {
                let capture_square = match side {
                    Side::White => Square::from_usize(target_square_index + 8),
                    Side::Black => Square::from_usize(target_square_index - 8),
                }
                .unwrap();

                game_clone
                    .mut_piece_bitboard(Piece::Pawn, opponent_side)
                    .pop_bit(capture_square);
            }
            MoveType::Castling => {
                let (a_file_square, c_file_square, d_file_square, f_file_square, h_file_square) =
                    match side {
                        Side::White => (Square::A1, Square::C1, Square::D1, Square::F1, Square::H1),
                        Side::Black => (Square::A8, Square::C8, Square::D8, Square::F8, Square::H8),
                    };

                if mv.target_square() == c_file_square {
                    game_clone
                        .mut_piece_bitboard(Piece::Rook, side)
                        .pop_bit(a_file_square);
                    game_clone
                        .mut_piece_bitboard(Piece::Rook, side)
                        .set_bit(d_file_square);
                } else {
                    game_clone
                        .mut_piece_bitboard(Piece::Rook, side)
                        .pop_bit(h_file_square);
                    game_clone
                        .mut_piece_bitboard(Piece::Rook, side)
                        .set_bit(f_file_square);
                }
            }
            _ => {}
        }

        if mv.move_type() != MoveType::DoublePawnPush {
            game_clone.en_passant_square = None;
        }

        if game_clone.castling_rights.0 != 0 {
            match side {
                Side::White => match mv.source_square() {
                    Square::A1 => game_clone
                        .castling_rights
                        .remove_castling_type(CastlingType::WhiteLong),
                    Square::E1 => {
                        game_clone
                            .castling_rights
                            .remove_castling_type(CastlingType::WhiteShort);
                        game_clone
                            .castling_rights
                            .remove_castling_type(CastlingType::WhiteLong);
                    }
                    Square::H1 => game_clone
                        .castling_rights
                        .remove_castling_type(CastlingType::WhiteShort),
                    _ => {}
                },
                Side::Black => match mv.source_square() {
                    Square::A8 => game_clone
                        .castling_rights
                        .remove_castling_type(CastlingType::BlackLong),
                    Square::E8 => {
                        game_clone
                            .castling_rights
                            .remove_castling_type(CastlingType::BlackShort);
                        game_clone
                            .castling_rights
                            .remove_castling_type(CastlingType::BlackLong);
                    }
                    Square::H8 => game_clone
                        .castling_rights
                        .remove_castling_type(CastlingType::BlackShort),
                    _ => {}
                },
            }
        }

        let king_square = game_clone
            .piece_bitboard(Piece::King, side)
            .get_lsb_square();

        if let Some(king_square) = king_square {
            let own_king_in_check = game_clone.is_square_attacked(opponent_side, king_square);

            if own_king_in_check {
                return Err(());
            }
        }

        *self = game_clone;
        self.side_to_move = side.opponent_side();

        Ok(())
    }

    pub fn is_square_attacked(&self, attacking_side: Side, square: Square) -> bool {
        for piece in Piece::iter() {
            let piece_attacks_square = attack_tables::ATTACK_TABLES.attack_table(
                self.board(None),
                piece,
                attacking_side.opponent_side(),
                square,
            ) & self.piece_bitboard(piece, attacking_side)
                != 0u64;

            if piece_attacks_square {
                return true;
            }
        }

        false
    }

    pub fn board(&self, side: Option<Side>) -> Bitboard {
        match side {
            Some(side) => match side {
                Side::White => {
                    self.white_pawns
                        | self.white_knights
                        | self.white_bishops
                        | self.white_rooks
                        | self.white_queens
                        | self.white_king
                }
                Side::Black => {
                    self.black_pawns
                        | self.black_knights
                        | self.black_bishops
                        | self.black_rooks
                        | self.black_queens
                        | self.black_king
                }
            },
            None => {
                self.white_pawns
                    | self.white_knights
                    | self.white_bishops
                    | self.white_rooks
                    | self.white_queens
                    | self.white_king
                    | self.black_pawns
                    | self.black_knights
                    | self.black_bishops
                    | self.black_rooks
                    | self.black_queens
                    | self.black_king
            }
        }
    }

    pub fn is_square_occupied(&self, square: Square) -> bool {
        self.board(None).bit_occupied(square)
    }

    pub fn side_to_move(&self) -> Side {
        self.side_to_move
    }

    pub fn en_passant_square(&self) -> Option<Square> {
        self.en_passant_square
    }

    pub fn castling_type_allowed(&self, castling_type: CastlingType) -> bool {
        self.castling_rights.0 & castling_type.to_u8().unwrap() != 0
    }

    pub fn piece_bitboard(&self, piece: Piece, side: Side) -> Bitboard {
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
        }
    }

    fn mut_piece_bitboard(&mut self, piece: Piece, side: Side) -> &mut Bitboard {
        match side {
            Side::White => match piece {
                Piece::Pawn => &mut self.white_pawns,
                Piece::Knight => &mut self.white_knights,
                Piece::Bishop => &mut self.white_bishops,
                Piece::Rook => &mut self.white_rooks,
                Piece::Queen => &mut self.white_queens,
                Piece::King => &mut self.white_king,
            },
            Side::Black => match piece {
                Piece::Pawn => &mut self.black_pawns,
                Piece::Knight => &mut self.black_knights,
                Piece::Bishop => &mut self.black_bishops,
                Piece::Rook => &mut self.black_rooks,
                Piece::Queen => &mut self.black_queens,
                Piece::King => &mut self.black_king,
            },
        }
    }

    fn _print(&self) {
        for square in Square::iter() {
            if square.file() == 0 {
                print!("{:<4}", (64 - square.to_usize().unwrap()) / 8);
            }

            match self._piece_at_square(square) {
                Some((piece, side)) => print!("{:<2}", piece._to_char(side)),
                None => print!(". "),
            }

            if square.file() == 7 {
                println!();
            }
        }

        println!();
        println!("    a b c d e f g h");
        println!();
        println!("Side to move: {:?}", self.side_to_move);
        println!("En passant square: {:?}", self.en_passant_square);
        println!("Castling rights: {}", self.castling_rights._as_string());
    }

    fn _piece_at_square(&self, square: Square) -> Option<(Piece, Side)> {
        for (bitboard, piece, side) in self._piece_bitboards() {
            if bitboard.bit_occupied(square) {
                return Some((piece, side));
            }
        }

        None
    }

    fn _piece_bitboards(&self) -> [(Bitboard, Piece, Side); 12] {
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
}

#[derive(Clone, Copy, PartialEq)]
pub struct Bitboard(u64);

impl Bitboard {
    pub fn new(bitboard: u64) -> Self {
        Bitboard(bitboard)
    }

    pub fn from_square(square: Square) -> Self {
        let mut bitboard = Self(0);

        bitboard.set_bit(square);

        bitboard
    }

    pub fn bit_occupied(&self, square: Square) -> bool {
        self.0 & (1 << square.to_usize().unwrap()) != 0
    }

    pub fn set_bit(&mut self, square: Square) {
        self.0 |= 1 << square.to_usize().unwrap();
    }

    pub fn pop_bit(&mut self, square: Square) {
        self.0 &= !(1 << square.to_usize().unwrap());
    }

    pub fn board(self) -> u64 {
        self.0
    }

    pub fn count_bits(self) -> u32 {
        self.0.count_ones()
    }

    // lsb = least significant bit
    pub fn get_lsb_square(self) -> Option<Square> {
        if self.0 == 0 {
            return None;
        }

        Square::from_u32(self.0.trailing_zeros())
    }

    fn _print(self) {
        for square in Square::iter() {
            if square.file() == 0 {
                print!("{:<4}", ((64 - square.to_usize().unwrap()) / 8));
            }

            print!("{:<2}", if self.bit_occupied(square) { 1 } else { 0 });

            if square.file() == 7 {
                println!();
            }
        }

        println!();
        println!("    a b c d e f g h");
        println!();
        println!("Bitboard decimal value: {}", self.0);
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl<T: Unsigned + AsPrimitive<u64>> BitAnd<T> for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: T) -> Self::Output {
        Self(self.0 & rhs.as_())
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = Self(self.0 & rhs.0)
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = Self(self.0 | rhs.0)
    }
}

impl<T: Unsigned + AsPrimitive<u64>> BitOrAssign<T> for Bitboard {
    fn bitor_assign(&mut self, rhs: T) {
        *self = Self(self.0 | rhs.as_())
    }
}

impl Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl<T: Unsigned + AsPrimitive<u64>> PartialEq<T> for Bitboard {
    fn eq(&self, other: &T) -> bool {
        self.0 == other.as_()
    }
}

impl<T: Unsigned + AsPrimitive<u64>> Shl<T> for Bitboard {
    type Output = Self;

    fn shl(self, rhs: T) -> Self::Output {
        Self(self.0 << rhs.as_())
    }
}

impl<T: Unsigned + AsPrimitive<u64>> Shr<T> for Bitboard {
    type Output = Self;

    fn shr(self, rhs: T) -> Self::Output {
        Self(self.0 >> rhs.as_())
    }
}

impl<T: Unsigned + AsPrimitive<u64>> ShrAssign<T> for Bitboard {
    fn shr_assign(&mut self, rhs: T) {
        *self = Self(self.0 >> rhs.as_())
    }
}

#[derive(Clone, Copy, Debug, Display, EnumIter, FromPrimitive, PartialEq, ToPrimitive)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Piece {
    pub fn _to_char(self, side: Side) -> char {
        match side {
            Side::White => match self {
                Self::Pawn => 'P',
                Self::Knight => 'N',
                Self::Bishop => 'B',
                Self::Rook => 'R',
                Self::Queen => 'Q',
                Self::King => 'K',
            },
            Side::Black => match self {
                Self::Pawn => 'p',
                Self::Knight => 'n',
                Self::Bishop => 'b',
                Self::Rook => 'r',
                Self::Queen => 'q',
                Self::King => 'k',
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Display, PartialEq)]
pub enum Side {
    White,
    Black,
}

impl Side {
    pub fn opponent_side(&self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

#[derive(
    Clone, Copy, Debug, Display, EnumIter, EnumString, FromPrimitive, PartialEq, ToPrimitive,
)]
pub enum Square {
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

impl Square {
    pub fn from_rank_file(rank: usize, file: usize) -> Self {
        Self::from_usize(rank * 8 + file).unwrap()
    }

    pub fn rank(&self) -> usize {
        self.to_usize().unwrap() / 8
    }

    pub fn file(&self) -> usize {
        self.to_usize().unwrap() % 8
    }

    pub fn _to_lowercase_string(&self) -> String {
        self.to_string().to_lowercase()
    }
}

#[derive(Clone, Copy, ToPrimitive)]
pub enum CastlingType {
    WhiteShort = 0b1000,
    WhiteLong = 0b0100,
    BlackShort = 0b0010,
    BlackLong = 0b0001,
}

impl CastlingType {
    pub fn _move_string(&self) -> &str {
        match self {
            Self::WhiteShort => "e1g1",
            Self::WhiteLong => "e1c1",
            Self::BlackShort => "e8g8",
            Self::BlackLong => "e8c8",
        }
    }
}

pub fn _perft_test(game: &mut Game, depth: u32) {
    let mut total_nodes = 0;
    let now = Instant::now();

    let moves = MoveList::generate_moves(game);

    println!("Move   Nodes   ");

    for mv in moves.move_list().iter().flatten() {
        let mut game_clone = game.clone();

        if game_clone.make_move(*mv, MoveFlag::All).is_err() {
            continue;
        }

        let mut nodes = 0;
        _perft(&mut game_clone, &mut nodes, depth - 1);

        print!("{:<6}", mv._to_string());
        print!("{:^7}", nodes);
        println!();

        total_nodes += nodes;
    }

    println!();
    println!("Depth: {}", depth);
    println!("Nodes: {}", total_nodes);
    println!("Time taken: {:?}", now.elapsed());
}

pub fn _perft(game: &mut Game, nodes: &mut u32, depth: u32) {
    if depth == 0 {
        *nodes += 1;
        return;
    }

    let moves = MoveList::generate_moves(game);

    for mv in moves.move_list().iter().flatten() {
        let mut game_clone = game.clone();

        if game_clone.make_move(*mv, MoveFlag::All).is_err() {
            continue;
        }

        _perft(&mut game_clone, nodes, depth - 1);
    }
}

#[derive(Clone)]
struct CastlingRights(u8);

impl CastlingRights {
    fn initialise(castling_rights_string: &str) -> Self {
        if castling_rights_string == "-" {
            return Self(0);
        };

        let mut castling_rights = 0;

        for character in castling_rights_string.chars() {
            match character {
                'K' => castling_rights |= CastlingType::WhiteShort.to_u8().unwrap(),
                'Q' => castling_rights |= CastlingType::WhiteLong.to_u8().unwrap(),
                'k' => castling_rights |= CastlingType::BlackShort.to_u8().unwrap(),
                'q' => castling_rights |= CastlingType::BlackLong.to_u8().unwrap(),
                _ => {}
            }
        }

        Self(castling_rights)
    }

    fn remove_castling_type(&mut self, castling_type: CastlingType) {
        self.0 &= !castling_type.to_u8().unwrap();
    }

    fn _as_string(&self) -> String {
        let mut castling_rights_string = String::new();

        if self.0 & CastlingType::WhiteShort.to_u8().unwrap() != 0 {
            castling_rights_string.push('K');
        }

        if self.0 & CastlingType::WhiteLong.to_u8().unwrap() != 0 {
            castling_rights_string.push('Q');
        }

        if self.0 & CastlingType::BlackShort.to_u8().unwrap() != 0 {
            castling_rights_string.push('k');
        }

        if self.0 & CastlingType::BlackLong.to_u8().unwrap() != 0 {
            castling_rights_string.push('q');
        }

        castling_rights_string
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn perft_start_position() {
        let mut game = Game::initialise("startpos");

        let mut nodes = 0;

        _perft(&mut game, &mut nodes, 6);

        assert_eq!(nodes, 119_060_324);
    }

    #[test]
    #[ignore]
    fn perft_tricky_position() {
        let mut game = Game::initialise(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        );

        let mut nodes = 0;

        _perft(&mut game, &mut nodes, 5);

        assert_eq!(nodes, 193_690_690);
    }

    #[test]
    fn tricky_position() {
        let game = Game::initialise(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        );

        let desired_white_pawns_bitboard = u64::pow(2, Square::A2 as u32)
            + u64::pow(2, Square::B2 as u32)
            + u64::pow(2, Square::C2 as u32)
            + u64::pow(2, Square::D5 as u32)
            + u64::pow(2, Square::E4 as u32)
            + u64::pow(2, Square::F2 as u32)
            + u64::pow(2, Square::G2 as u32)
            + u64::pow(2, Square::H2 as u32);
        let desired_white_knights_bitboard =
            u64::pow(2, Square::C3 as u32) + u64::pow(2, Square::E5 as u32);
        let desired_white_bishops_bitboard =
            u64::pow(2, Square::D2 as u32) + u64::pow(2, Square::E2 as u32);
        let desired_white_rooks_bitboard =
            u64::pow(2, Square::A1 as u32) + u64::pow(2, Square::H1 as u32);
        let desired_white_queens_bitboard = u64::pow(2, Square::F3 as u32);
        let desired_white_king_bitboard = u64::pow(2, Square::E1 as u32);

        let desired_black_pawns_bitboard = u64::pow(2, Square::A7 as u32)
            + u64::pow(2, Square::B4 as u32)
            + u64::pow(2, Square::C7 as u32)
            + u64::pow(2, Square::D7 as u32)
            + u64::pow(2, Square::E6 as u32)
            + u64::pow(2, Square::F7 as u32)
            + u64::pow(2, Square::G6 as u32)
            + u64::pow(2, Square::H3 as u32);
        let desired_black_knights_bitboard =
            u64::pow(2, Square::B6 as u32) + u64::pow(2, Square::F6 as u32);
        let desired_black_bishops_bitboard =
            u64::pow(2, Square::A6 as u32) + u64::pow(2, Square::G7 as u32);
        let desired_black_rooks_bitboard =
            u64::pow(2, Square::A8 as u32) + u64::pow(2, Square::H8 as u32);
        let desired_black_queens_bitboard = u64::pow(2, Square::E7 as u32);
        let desired_black_king_bitboard = u64::pow(2, Square::E8 as u32);

        assert_eq!(game.white_pawns.0, desired_white_pawns_bitboard);
        assert_eq!(game.white_knights.0, desired_white_knights_bitboard);
        assert_eq!(game.white_bishops.0, desired_white_bishops_bitboard);
        assert_eq!(game.white_rooks.0, desired_white_rooks_bitboard);
        assert_eq!(game.white_queens.0, desired_white_queens_bitboard);
        assert_eq!(game.white_king.0, desired_white_king_bitboard);
        assert_eq!(game.black_pawns.0, desired_black_pawns_bitboard);
        assert_eq!(game.black_knights.0, desired_black_knights_bitboard);
        assert_eq!(game.black_bishops.0, desired_black_bishops_bitboard);
        assert_eq!(game.black_rooks.0, desired_black_rooks_bitboard);
        assert_eq!(game.black_queens.0, desired_black_queens_bitboard);
        assert_eq!(game.black_king.0, desired_black_king_bitboard);
    }

    #[test]
    fn killer_position() {
        let game =
            Game::initialise("rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/3P3P/P1P1P3/RNBQKBNR w KQkq e6 0 1");

        let desired_white_pawns_bitboard = u64::pow(2, Square::A2 as u32)
            + u64::pow(2, Square::B4 as u32)
            + u64::pow(2, Square::C2 as u32)
            + u64::pow(2, Square::D3 as u32)
            + u64::pow(2, Square::D4 as u32)
            + u64::pow(2, Square::E2 as u32)
            + u64::pow(2, Square::F5 as u32)
            + u64::pow(2, Square::G7 as u32)
            + u64::pow(2, Square::H3 as u32);
        let desired_white_knights_bitboard =
            u64::pow(2, Square::B1 as u32) + u64::pow(2, Square::G1 as u32);
        let desired_white_bishops_bitboard =
            u64::pow(2, Square::C1 as u32) + u64::pow(2, Square::F1 as u32);
        let desired_white_rooks_bitboard =
            u64::pow(2, Square::A1 as u32) + u64::pow(2, Square::H1 as u32);
        let desired_white_queens_bitboard = u64::pow(2, Square::D1 as u32);
        let desired_white_king_bitboard = u64::pow(2, Square::E1 as u32);

        let desired_black_pawns_bitboard = u64::pow(2, Square::A7 as u32)
            + u64::pow(2, Square::B7 as u32)
            + u64::pow(2, Square::C5 as u32)
            + u64::pow(2, Square::D7 as u32)
            + u64::pow(2, Square::E5 as u32)
            + u64::pow(2, Square::F7 as u32)
            + u64::pow(2, Square::H7 as u32);
        let desired_black_knights_bitboard = u64::pow(2, Square::B8 as u32);
        let desired_black_bishops_bitboard =
            u64::pow(2, Square::C8 as u32) + u64::pow(2, Square::F8 as u32);
        let desired_black_rooks_bitboard =
            u64::pow(2, Square::A8 as u32) + u64::pow(2, Square::H8 as u32);
        let desired_black_queens_bitboard = u64::pow(2, Square::D8 as u32);
        let desired_black_king_bitboard = u64::pow(2, Square::E8 as u32);

        assert_eq!(game.white_pawns.0, desired_white_pawns_bitboard);
        assert_eq!(game.white_knights.0, desired_white_knights_bitboard);
        assert_eq!(game.white_bishops.0, desired_white_bishops_bitboard);
        assert_eq!(game.white_rooks.0, desired_white_rooks_bitboard);
        assert_eq!(game.white_queens.0, desired_white_queens_bitboard);
        assert_eq!(game.white_king.0, desired_white_king_bitboard);
        assert_eq!(game.black_pawns.0, desired_black_pawns_bitboard);
        assert_eq!(game.black_knights.0, desired_black_knights_bitboard);
        assert_eq!(game.black_bishops.0, desired_black_bishops_bitboard);
        assert_eq!(game.black_rooks.0, desired_black_rooks_bitboard);
        assert_eq!(game.black_queens.0, desired_black_queens_bitboard);
        assert_eq!(game.black_king.0, desired_black_king_bitboard);
    }

    #[test]
    fn cmk_position() {
        let game = Game::initialise(
            "r2q1rk1/ppp2ppp/2n1bn2/2b1p3/3pP3/3P1NPP/PPP1NPB1/R1BQ1RK1 b - - 0 9",
        );

        let desired_white_pawns_bitboard = u64::pow(2, Square::A2 as u32)
            + u64::pow(2, Square::B2 as u32)
            + u64::pow(2, Square::C2 as u32)
            + u64::pow(2, Square::D3 as u32)
            + u64::pow(2, Square::E4 as u32)
            + u64::pow(2, Square::F2 as u32)
            + u64::pow(2, Square::G3 as u32)
            + u64::pow(2, Square::H3 as u32);
        let desired_white_knights_bitboard =
            u64::pow(2, Square::E2 as u32) + u64::pow(2, Square::F3 as u32);
        let desired_white_bishops_bitboard =
            u64::pow(2, Square::C1 as u32) + u64::pow(2, Square::G2 as u32);
        let desired_white_rooks_bitboard =
            u64::pow(2, Square::A1 as u32) + u64::pow(2, Square::F1 as u32);
        let desired_white_queens_bitboard = u64::pow(2, Square::D1 as u32);
        let desired_white_king_bitboard = u64::pow(2, Square::G1 as u32);

        let desired_black_pawns_bitboard = u64::pow(2, Square::A7 as u32)
            + u64::pow(2, Square::B7 as u32)
            + u64::pow(2, Square::C7 as u32)
            + u64::pow(2, Square::D4 as u32)
            + u64::pow(2, Square::E5 as u32)
            + u64::pow(2, Square::F7 as u32)
            + u64::pow(2, Square::G7 as u32)
            + u64::pow(2, Square::H7 as u32);
        let desired_black_knights_bitboard =
            u64::pow(2, Square::C6 as u32) + u64::pow(2, Square::F6 as u32);
        let desired_black_bishops_bitboard =
            u64::pow(2, Square::C5 as u32) + u64::pow(2, Square::E6 as u32);
        let desired_black_rooks_bitboard =
            u64::pow(2, Square::A8 as u32) + u64::pow(2, Square::F8 as u32);
        let desired_black_queens_bitboard = u64::pow(2, Square::D8 as u32);
        let desired_black_king_bitboard = u64::pow(2, Square::G8 as u32);

        assert_eq!(game.white_pawns.0, desired_white_pawns_bitboard);
        assert_eq!(game.white_knights.0, desired_white_knights_bitboard);
        assert_eq!(game.white_bishops.0, desired_white_bishops_bitboard);
        assert_eq!(game.white_rooks.0, desired_white_rooks_bitboard);
        assert_eq!(game.white_queens.0, desired_white_queens_bitboard);
        assert_eq!(game.white_king.0, desired_white_king_bitboard);
        assert_eq!(game.black_pawns.0, desired_black_pawns_bitboard);
        assert_eq!(game.black_knights.0, desired_black_knights_bitboard);
        assert_eq!(game.black_bishops.0, desired_black_bishops_bitboard);
        assert_eq!(game.black_rooks.0, desired_black_rooks_bitboard);
        assert_eq!(game.black_queens.0, desired_black_queens_bitboard);
        assert_eq!(game.black_king.0, desired_black_king_bitboard);
    }

    #[test]
    fn set_bit() {
        let mut bitboard1 = Bitboard(0);
        let mut bitboard2 = Bitboard(0);
        let mut bitboard3 = Bitboard(0);

        bitboard1.set_bit(Square::H2);
        bitboard2.set_bit(Square::G6);
        bitboard3.set_bit(Square::B4);

        assert_eq!(
            bitboard1.0,
            u64::pow(2, Square::H2.to_usize().unwrap() as u32)
        );
        assert_eq!(
            bitboard2.0,
            u64::pow(2, Square::G6.to_usize().unwrap() as u32)
        );
        assert_eq!(
            bitboard3.0,
            u64::pow(2, Square::B4.to_usize().unwrap() as u32)
        );
    }

    #[test]
    fn pop_bit() {
        let mut bitboard1 = Bitboard(0);
        let mut bitboard2 = Bitboard(0);
        let mut bitboard3 = Bitboard(0);

        bitboard1.set_bit(Square::G5);
        bitboard1.set_bit(Square::A8);
        bitboard1.pop_bit(Square::G5);

        bitboard2.set_bit(Square::C1);
        bitboard2.set_bit(Square::A7);
        bitboard2.pop_bit(Square::C1);

        bitboard3.set_bit(Square::C4);
        bitboard3.set_bit(Square::B8);
        bitboard3.pop_bit(Square::C4);

        assert_eq!(
            bitboard1.0,
            u64::pow(2, Square::A8.to_usize().unwrap() as u32)
        );
        assert_eq!(
            bitboard2.0,
            u64::pow(2, Square::A7.to_usize().unwrap() as u32)
        );
        assert_eq!(
            bitboard3.0,
            u64::pow(2, Square::B8.to_usize().unwrap() as u32)
        );
    }

    #[test]
    fn pop_unset_bit() {
        let mut bitboard1 = Bitboard(0);
        let mut bitboard2 = Bitboard(0);

        bitboard1.set_bit(Square::F1);
        bitboard1.pop_bit(Square::F1);
        bitboard1.pop_bit(Square::F1);

        bitboard2.pop_bit(Square::G2);

        assert_eq!(bitboard1.0, 0);
        assert_eq!(bitboard2.0, 0);
    }
}
