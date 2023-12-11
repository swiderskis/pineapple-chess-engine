use std::str::FromStr;

use super::{
    attack_tables::AttackTables,
    game::{Bitboard, CastlingType, Game, Piece, Side, Square},
    InputError,
};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use strum::{IntoEnumIterator, ParseError};

const PROMOTION_PIECES: [Piece; 4] = [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen];

#[derive(Clone)]
pub struct MoveList(Vec<Move>);

impl MoveList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn generate_moves(game: &Game, attack_tables: &AttackTables) -> Self {
        let mut move_list = Self::new();

        let side = game.side_to_move();

        for piece in Piece::iter() {
            let mut bitboard = game.piece_bitboard(piece, side);

            while let Some(source_square) = bitboard.get_lsb_square() {
                match piece {
                    Piece::Pawn => {
                        move_list.generate_pawn_moves(game, source_square, attack_tables)
                    }
                    _ => move_list.generate_piece_moves(game, piece, source_square, attack_tables),
                };

                bitboard.pop_bit(source_square);
            }
        }

        move_list
    }

    pub fn find_move_from_string(&self, move_string: &str) -> Result<Move, InputError> {
        match Self::parse_move_string(move_string) {
            Ok(move_search) => {
                let mv = self.find_move(move_search)?;

                Ok(mv)
            }
            Err(_) => Err(InputError::InvalidMoveString),
        }
    }

    pub fn find_move(&self, move_search: MoveSearch) -> Result<Move, InputError> {
        for mv in &self.0 {
            if mv.source_square() == move_search.source_square
                && mv.target_square() == move_search.target_square
                && mv.promoted_piece() == move_search.promoted_piece
            {
                return Ok(mv.clone());
            }
        }

        Err(InputError::IllegalMove)
    }

    pub fn vec(&self) -> &Vec<Move> {
        &self.0
    }

    pub fn mut_vec(&mut self) -> &mut Vec<Move> {
        &mut self.0
    }

    fn generate_pawn_moves(
        &mut self,
        game: &Game,
        source_square: Square,
        attack_tables: &AttackTables,
    ) {
        let side = game.side_to_move();

        let source_square_index = source_square as usize;
        let target_square = match side {
            Side::White => Square::from_usize(source_square_index - 8),
            Side::Black => Square::from_usize(source_square_index + 8),
        }
        .unwrap();

        let second_rank = Bitboard::new(0xFF_0000_0000_0000);
        let seventh_rank = Bitboard::new(0xFF00);

        let single_piece = Bitboard::from_square(source_square);

        let pawn_on_second_rank = second_rank & single_piece != 0u64;
        let pawn_on_seventh_rank = seventh_rank & single_piece != 0u64;

        let pawn_ready_to_promote = (side == Side::White && pawn_on_seventh_rank)
            || (side == Side::Black && pawn_on_second_rank);

        if pawn_ready_to_promote && !game.is_square_occupied(target_square) {
            for promoted_piece in PROMOTION_PIECES {
                self.0.push(Move::new(
                    source_square,
                    target_square,
                    Piece::Pawn,
                    Some(promoted_piece),
                    MoveType::Quiet,
                ));
            }
        } else if !game.is_square_occupied(target_square) {
            self.0.push(Move::new(
                source_square,
                target_square,
                Piece::Pawn,
                None,
                MoveType::Quiet,
            ));

            let double_push_target_square = if side == Side::White && pawn_on_second_rank {
                Some(Square::from_usize(source_square_index - 16).unwrap())
            } else if side == Side::Black && pawn_on_seventh_rank {
                Some(Square::from_usize(source_square_index + 16).unwrap())
            } else {
                None
            };

            if let Some(target_square) = double_push_target_square {
                if !game.is_square_occupied(target_square) {
                    self.0.push(Move::new(
                        source_square,
                        target_square,
                        Piece::Pawn,
                        None,
                        MoveType::DoublePawnPush,
                    ));
                }
            }
        }

        let mut attacks = Self::generate_attacks(game, Piece::Pawn, source_square, attack_tables);

        while let Some(target_square) = attacks.get_lsb_square() {
            if pawn_ready_to_promote {
                for promoted_piece in PROMOTION_PIECES {
                    self.0.push(Move::new(
                        source_square,
                        target_square,
                        Piece::Pawn,
                        Some(promoted_piece),
                        MoveType::Capture,
                    ));
                }
            } else {
                self.0.push(Move::new(
                    source_square,
                    target_square,
                    Piece::Pawn,
                    None,
                    MoveType::Capture,
                ));
            }

            attacks.pop_bit(target_square);
        }

        let attack_table =
            attack_tables.attack_table(game.board(None), Piece::Pawn, side, source_square);

        if let Some(target_square) = game.en_passant_square() {
            let en_passant_square_attacked =
                attack_table & Bitboard::from_square(target_square) != 0u64;

            if en_passant_square_attacked {
                self.0.push(Move::new(
                    source_square,
                    target_square,
                    Piece::Pawn,
                    None,
                    MoveType::EnPassant,
                ));
            }
        }
    }

    fn generate_piece_moves(
        &mut self,
        game: &Game,
        piece: Piece,
        source_square: Square,
        attack_tables: &AttackTables,
    ) {
        let mut attacks = Self::generate_attacks(game, piece, source_square, attack_tables);

        while let Some(target_square) = attacks.get_lsb_square() {
            let move_type = if game.is_square_occupied(target_square) {
                MoveType::Capture
            } else {
                MoveType::Quiet
            };

            self.0.push(Move::new(
                source_square,
                target_square,
                piece,
                None,
                move_type,
            ));

            attacks.pop_bit(target_square);
        }

        if piece == Piece::King {
            self.generate_castling_moves(game, attack_tables);
        }
    }

    fn generate_castling_moves(&mut self, game: &Game, attack_tables: &AttackTables) {
        let side = game.side_to_move();
        let opponent_side = side.opponent_side();

        match side {
            Side::White => {
                if game.castling_type_allowed(CastlingType::WhiteShort)
                    && !game.is_square_occupied(Square::F1)
                    && !game.is_square_occupied(Square::G1)
                    && !game.is_square_attacked(attack_tables, opponent_side, Square::E1)
                    && !game.is_square_attacked(attack_tables, opponent_side, Square::F1)
                {
                    self.0.push(Move::new(
                        Square::E1,
                        Square::G1,
                        Piece::King,
                        None,
                        MoveType::Castling,
                    ));
                }

                if game.castling_type_allowed(CastlingType::WhiteLong)
                    && !game.is_square_occupied(Square::B1)
                    && !game.is_square_occupied(Square::C1)
                    && !game.is_square_occupied(Square::D1)
                    && !game.is_square_attacked(attack_tables, opponent_side, Square::D1)
                    && !game.is_square_attacked(attack_tables, opponent_side, Square::E1)
                {
                    self.0.push(Move::new(
                        Square::E1,
                        Square::C1,
                        Piece::King,
                        None,
                        MoveType::Castling,
                    ));
                }
            }
            Side::Black => {
                if game.castling_type_allowed(CastlingType::BlackShort)
                    && !game.is_square_occupied(Square::F8)
                    && !game.is_square_occupied(Square::G8)
                    && !game.is_square_attacked(attack_tables, opponent_side, Square::E8)
                    && !game.is_square_attacked(attack_tables, opponent_side, Square::F8)
                {
                    self.0.push(Move::new(
                        Square::E8,
                        Square::G8,
                        Piece::King,
                        None,
                        MoveType::Castling,
                    ));
                }

                if game.castling_type_allowed(CastlingType::BlackLong)
                    && !game.is_square_occupied(Square::B8)
                    && !game.is_square_occupied(Square::C8)
                    && !game.is_square_occupied(Square::D8)
                    && !game.is_square_attacked(attack_tables, opponent_side, Square::D8)
                    && !game.is_square_attacked(attack_tables, opponent_side, Square::E8)
                {
                    self.0.push(Move::new(
                        Square::E8,
                        Square::C8,
                        Piece::King,
                        None,
                        MoveType::Castling,
                    ));
                }
            }
        }
    }

    fn generate_attacks(
        game: &Game,
        piece: Piece,
        source_square: Square,
        attack_tables: &AttackTables,
    ) -> Bitboard {
        let attack_table =
            attack_tables.attack_table(game.board(None), piece, game.side_to_move(), source_square);
        let valid_attack_squares = match piece {
            Piece::Pawn => game.board(Some(game.side_to_move().opponent_side())),
            _ => !game.board(Some(game.side_to_move())),
        };

        attack_table & valid_attack_squares
    }

    fn parse_move_string(move_string: &str) -> Result<MoveSearch, ParseError> {
        let (source_square_string, remaining_move_string) = move_string.split_at(2);
        let (target_square_string, promoted_piece_string) = remaining_move_string.split_at(2);

        let source_square = Square::from_str(source_square_string.to_uppercase().as_str())?;
        let target_square = Square::from_str(target_square_string.to_uppercase().as_str())?;

        let promoted_piece = if let Some(promoted_piece_char) = promoted_piece_string.chars().nth(0)
        {
            Some(Piece::from_char(promoted_piece_char)?)
        } else {
            None
        };

        let move_search = MoveSearch::new(source_square, target_square, promoted_piece);

        Ok(move_search)
    }

    pub fn _length(&self) -> usize {
        self.0.len()
    }
}

#[derive(Clone, Copy, Debug, FromPrimitive, PartialEq)]
pub enum MoveType {
    Quiet,
    Capture,
    DoublePawnPush,
    EnPassant,
    Castling,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Move {
    source_square: Square,
    target_square: Square,
    piece: Piece,
    promoted_piece: Option<Piece>,
    move_type: MoveType,
}

impl Move {
    fn new(
        source_square: Square,
        target_square: Square,
        piece: Piece,
        promoted_piece: Option<Piece>,
        move_type: MoveType,
    ) -> Self {
        Self {
            source_square,
            target_square,
            piece,
            promoted_piece,
            move_type,
        }
    }

    pub fn source_square(&self) -> Square {
        self.source_square
    }

    pub fn target_square(&self) -> Square {
        self.target_square
    }

    pub fn piece(&self) -> Piece {
        self.piece
    }

    pub fn promoted_piece(&self) -> Option<Piece> {
        self.promoted_piece
    }

    pub fn move_type(&self) -> MoveType {
        self.move_type
    }

    pub fn as_string(&self) -> String {
        let source_square_string = self.source_square._to_lowercase_string();
        let target_square_string = self.target_square._to_lowercase_string();

        match self.promoted_piece {
            Some(promoted_piece) => {
                let promoted_piece_string = promoted_piece._to_char(None).to_string();

                source_square_string + &target_square_string + &promoted_piece_string
            }
            None => source_square_string + &target_square_string,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct MoveSearch {
    source_square: Square,
    target_square: Square,
    promoted_piece: Option<Piece>,
}

impl MoveSearch {
    pub fn new(
        source_square: Square,
        target_square: Square,
        promoted_piece: Option<Piece>,
    ) -> Self {
        Self {
            source_square,
            target_square,
            promoted_piece,
        }
    }

    pub fn _as_string(&self) -> String {
        let source_square_string = self.source_square._to_lowercase_string();
        let target_square_string = self.target_square._to_lowercase_string();

        match self.promoted_piece {
            Some(promoted_piece) => {
                let promoted_piece_string = promoted_piece._to_char(None).to_string();

                source_square_string + &target_square_string + &promoted_piece_string
            }
            None => source_square_string + &target_square_string,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unusual_byte_groupings)]
mod tests {
    use super::*;

    #[test]
    fn single_pawn_push() {
        let mut white_game = Game::initialise();
        let mut black_game = Game::initialise();

        let attack_tables = AttackTables::initialise();

        white_game.load_fen("8/8/8/8/8/3P4/8/8 w - - 0 1").unwrap();
        black_game.load_fen("8/8/3p4/8/8/8/8/8 b - - 0 1").unwrap();

        let white_square = Square::D3;
        let black_square = Square::D6;

        let mut white_move_list = MoveList::new();
        let mut black_move_list = MoveList::new();

        white_move_list.generate_pawn_moves(&white_game, white_square, &attack_tables);
        black_move_list.generate_pawn_moves(&black_game, black_square, &attack_tables);

        let white_pawn_push = Move::new(Square::D3, Square::D4, Piece::Pawn, None, MoveType::Quiet);
        let black_pawn_push = Move::new(Square::D6, Square::D5, Piece::Pawn, None, MoveType::Quiet);

        let white_moves_correct =
            white_move_list.0.contains(&white_pawn_push) && white_move_list.0.len() == 1;
        let black_moves_correct =
            black_move_list.0.contains(&black_pawn_push) && black_move_list.0.len() == 1;

        assert!(white_moves_correct);
        assert!(black_moves_correct);

        // blocked from pushing
        white_game
            .load_fen("8/8/8/8/3p4/3P4/8/8 w - - 0 1")
            .unwrap();
        black_game
            .load_fen("8/8/3p4/3P4/8/8/8/8 b - - 0 1")
            .unwrap();

        let white_square = Square::D3;
        let black_square = Square::D6;

        let mut white_move_list = MoveList::new();
        let mut black_move_list = MoveList::new();

        white_move_list.generate_pawn_moves(&white_game, white_square, &attack_tables);
        black_move_list.generate_pawn_moves(&black_game, black_square, &attack_tables);

        let white_moves_correct = white_move_list.0.is_empty();
        let black_moves_correct = black_move_list.0.is_empty();

        assert!(white_moves_correct);
        assert!(black_moves_correct);
    }

    #[test]
    fn double_pawn_push() {
        let mut white_game = Game::initialise();
        let mut black_game = Game::initialise();

        let attack_tables = AttackTables::initialise();

        white_game.load_fen("8/8/8/8/8/8/3P4/8 w - - 0 1").unwrap();
        black_game.load_fen("8/3p4/8/8/8/8/8/8 b - - 0 1").unwrap();

        let white_square = Square::D2;
        let black_square = Square::D7;

        let mut white_move_list = MoveList::new();
        let mut black_move_list = MoveList::new();

        white_move_list.generate_pawn_moves(&white_game, white_square, &attack_tables);
        black_move_list.generate_pawn_moves(&black_game, black_square, &attack_tables);

        let white_single_pawn_push =
            Move::new(Square::D2, Square::D3, Piece::Pawn, None, MoveType::Quiet);
        let white_double_pawn_push = Move::new(
            Square::D2,
            Square::D4,
            Piece::Pawn,
            None,
            MoveType::DoublePawnPush,
        );

        let black_single_pawn_push =
            Move::new(Square::D7, Square::D6, Piece::Pawn, None, MoveType::Quiet);
        let black_double_pawn_push = Move::new(
            Square::D7,
            Square::D5,
            Piece::Pawn,
            None,
            MoveType::DoublePawnPush,
        );

        let white_moves_correct = white_move_list.0.contains(&white_single_pawn_push)
            && white_move_list.0.contains(&white_double_pawn_push)
            && white_move_list.0.len() == 2;
        let black_moves_correct = black_move_list.0.contains(&black_single_pawn_push)
            && black_move_list.0.contains(&black_double_pawn_push)
            && black_move_list.0.len() == 2;

        assert!(white_moves_correct);
        assert!(black_moves_correct);

        // blocked from double push
        white_game
            .load_fen("8/8/8/8/3p4/8/3P4/8 w - - 0 1")
            .unwrap();
        black_game
            .load_fen("8/3p4/8/3P4/8/8/8/8 b - - 0 1")
            .unwrap();

        let mut white_move_list = MoveList::new();
        let mut black_move_list = MoveList::new();

        white_move_list.generate_pawn_moves(&white_game, white_square, &attack_tables);
        black_move_list.generate_pawn_moves(&black_game, black_square, &attack_tables);

        let white_single_pawn_push =
            Move::new(Square::D2, Square::D3, Piece::Pawn, None, MoveType::Quiet);
        let black_single_pawn_push =
            Move::new(Square::D7, Square::D6, Piece::Pawn, None, MoveType::Quiet);

        let white_moves_correct =
            white_move_list.0.contains(&white_single_pawn_push) && white_move_list.0.len() == 1;
        let black_moves_correct =
            black_move_list.0.contains(&black_single_pawn_push) && black_move_list.0.len() == 1;

        assert!(white_moves_correct);
        assert!(black_moves_correct);

        // blocked from single push
        white_game
            .load_fen("8/8/8/8/8/3p4/3P4/8 w - - 0 1")
            .unwrap();
        black_game
            .load_fen("8/3p4/3P4/8/8/8/8/8 b - - 0 1")
            .unwrap();

        let mut white_move_list = MoveList::new();
        let mut black_move_list = MoveList::new();

        white_move_list.generate_pawn_moves(&white_game, white_square, &attack_tables);
        black_move_list.generate_pawn_moves(&black_game, black_square, &attack_tables);

        let white_moves_correct = white_move_list.0.is_empty();
        let black_moves_correct = black_move_list.0.is_empty();

        assert!(white_moves_correct);
        assert!(black_moves_correct);
    }

    #[test]
    fn pawn_capture() {
        let mut white_game = Game::initialise();
        let mut black_game = Game::initialise();

        let attack_tables = AttackTables::initialise();

        white_game
            .load_fen("8/8/8/2P1p3/3P4/8/8/8 w - - 0 1")
            .unwrap();
        black_game
            .load_fen("8/8/8/3p4/2p1P3/8/8/8 b - - 0 1")
            .unwrap();

        let white_square = Square::D4;
        let black_square = Square::D5;

        let mut white_move_list = MoveList::new();
        let mut black_move_list = MoveList::new();

        white_move_list.generate_pawn_moves(&white_game, white_square, &attack_tables);
        black_move_list.generate_pawn_moves(&black_game, black_square, &attack_tables);

        let white_capture = Move::new(Square::D4, Square::E5, Piece::Pawn, None, MoveType::Capture);
        let black_capture = Move::new(Square::D5, Square::E4, Piece::Pawn, None, MoveType::Capture);

        assert!(white_move_list.0.contains(&white_capture));
        assert!(black_move_list.0.contains(&black_capture));
    }

    #[test]
    fn pawn_promotion() {
        let mut white_game = Game::initialise();
        let mut black_game = Game::initialise();

        let attack_tables = AttackTables::initialise();

        white_game.load_fen("8/3P4/8/8/8/8/8/8 w - - 0 1").unwrap();
        black_game.load_fen("8/8/8/8/8/8/3p4/8 b - - 0 1").unwrap();

        let white_square = Square::D7;
        let black_square = Square::D2;

        let mut white_move_list = MoveList::new();
        let mut black_move_list = MoveList::new();

        white_move_list.generate_pawn_moves(&white_game, white_square, &attack_tables);
        black_move_list.generate_pawn_moves(&black_game, black_square, &attack_tables);

        let white_promotion_queen = Move::new(
            Square::D7,
            Square::D8,
            Piece::Pawn,
            Some(Piece::Queen),
            MoveType::Quiet,
        );
        let white_promotion_rook = Move::new(
            Square::D7,
            Square::D8,
            Piece::Pawn,
            Some(Piece::Rook),
            MoveType::Quiet,
        );
        let white_promotion_bishop = Move::new(
            Square::D7,
            Square::D8,
            Piece::Pawn,
            Some(Piece::Bishop),
            MoveType::Quiet,
        );
        let white_promotion_knight = Move::new(
            Square::D7,
            Square::D8,
            Piece::Pawn,
            Some(Piece::Knight),
            MoveType::Quiet,
        );

        let black_promotion_queen = Move::new(
            Square::D2,
            Square::D1,
            Piece::Pawn,
            Some(Piece::Queen),
            MoveType::Quiet,
        );
        let black_promotion_rook = Move::new(
            Square::D2,
            Square::D1,
            Piece::Pawn,
            Some(Piece::Rook),
            MoveType::Quiet,
        );
        let black_promotion_bishop = Move::new(
            Square::D2,
            Square::D1,
            Piece::Pawn,
            Some(Piece::Bishop),
            MoveType::Quiet,
        );
        let black_promotion_knight = Move::new(
            Square::D2,
            Square::D1,
            Piece::Pawn,
            Some(Piece::Knight),
            MoveType::Quiet,
        );

        let white_moves_correct = white_move_list.0.contains(&white_promotion_queen)
            && white_move_list.0.contains(&white_promotion_rook)
            && white_move_list.0.contains(&white_promotion_bishop)
            && white_move_list.0.contains(&white_promotion_knight)
            && white_move_list.0.len() == 4;
        let black_moves_correct = black_move_list.0.contains(&black_promotion_queen)
            && black_move_list.0.contains(&black_promotion_rook)
            && black_move_list.0.contains(&black_promotion_bishop)
            && black_move_list.0.contains(&black_promotion_knight)
            && black_move_list.0.len() == 4;

        assert!(white_moves_correct);
        assert!(black_moves_correct);
    }

    #[test]
    fn en_passant() {
        let mut white_game = Game::initialise();
        let mut black_game = Game::initialise();

        let attack_tables = AttackTables::initialise();

        white_game
            .load_fen("8/8/8/3Pp3/8/8/8/8 w - e6 0 1")
            .unwrap();
        black_game
            .load_fen("8/8/8/8/3pP3/8/8/8 b - e3 0 1")
            .unwrap();

        let white_square = Square::D5;
        let black_square = Square::D4;

        let mut white_move_list = MoveList::new();
        let mut black_move_list = MoveList::new();

        white_move_list.generate_pawn_moves(&white_game, white_square, &attack_tables);
        black_move_list.generate_pawn_moves(&black_game, black_square, &attack_tables);

        let white_en_passant = Move::new(
            Square::D5,
            Square::E6,
            Piece::Pawn,
            None,
            MoveType::EnPassant,
        );
        let black_en_passant = Move::new(
            Square::D4,
            Square::E3,
            Piece::Pawn,
            None,
            MoveType::EnPassant,
        );

        let white_moves_correct = white_move_list.0.contains(&white_en_passant);
        let black_moves_correct = black_move_list.0.contains(&black_en_passant);

        assert!(white_moves_correct);
        assert!(black_moves_correct);
    }

    #[test]
    fn knight_moves() {
        let mut game = Game::initialise();
        let attack_tables = AttackTables::initialise();

        game.load_fen("8/8/2p5/5P2/3N4/1p6/2p1P3/8 w - - 0 1")
            .unwrap();

        let mut move_list = MoveList::new();

        move_list.generate_piece_moves(&game, Piece::Knight, Square::D4, &attack_tables);

        let desired_c6_move = Move::new(
            Square::D4,
            Square::C6,
            Piece::Knight,
            None,
            MoveType::Capture,
        );
        let desired_e6_move =
            Move::new(Square::D4, Square::E6, Piece::Knight, None, MoveType::Quiet);
        let desired_b5_move =
            Move::new(Square::D4, Square::B5, Piece::Knight, None, MoveType::Quiet);
        let desired_b3_move = Move::new(
            Square::D4,
            Square::B3,
            Piece::Knight,
            None,
            MoveType::Capture,
        );
        let desired_f3_move =
            Move::new(Square::D4, Square::F3, Piece::Knight, None, MoveType::Quiet);
        let desired_c2_move = Move::new(
            Square::D4,
            Square::C2,
            Piece::Knight,
            None,
            MoveType::Capture,
        );

        let knight_moves_correct = move_list.0.contains(&desired_c6_move)
            && move_list.0.contains(&desired_e6_move)
            && move_list.0.contains(&desired_b5_move)
            && move_list.0.contains(&desired_b3_move)
            && move_list.0.contains(&desired_f3_move)
            && move_list.0.contains(&desired_c2_move)
            && move_list.0.len() == 6;

        assert!(knight_moves_correct);
    }

    #[test]
    fn bishop_moves() {
        let mut game = Game::initialise();
        let attack_tables = AttackTables::initialise();

        game.load_fen("8/6p1/8/8/3B4/8/5P2/8 w - - 0 1").unwrap();

        let source_square = Square::D4;

        let mut move_list = MoveList::new();

        move_list.generate_piece_moves(&game, Piece::Bishop, source_square, &attack_tables);

        let desired_a7_move =
            Move::new(Square::D4, Square::A7, Piece::Bishop, None, MoveType::Quiet);
        let desired_b6_move =
            Move::new(Square::D4, Square::B6, Piece::Bishop, None, MoveType::Quiet);
        let desired_c5_move =
            Move::new(Square::D4, Square::C5, Piece::Bishop, None, MoveType::Quiet);

        let desired_g7_move = Move::new(
            Square::D4,
            Square::G7,
            Piece::Bishop,
            None,
            MoveType::Capture,
        );
        let desired_f6_move =
            Move::new(Square::D4, Square::F6, Piece::Bishop, None, MoveType::Quiet);
        let desired_e5_move =
            Move::new(Square::D4, Square::E5, Piece::Bishop, None, MoveType::Quiet);

        let desired_c3_move =
            Move::new(Square::D4, Square::C3, Piece::Bishop, None, MoveType::Quiet);
        let desired_b2_move =
            Move::new(Square::D4, Square::B2, Piece::Bishop, None, MoveType::Quiet);
        let desired_a1_move =
            Move::new(Square::D4, Square::A1, Piece::Bishop, None, MoveType::Quiet);

        let desired_e3_move =
            Move::new(Square::D4, Square::E3, Piece::Bishop, None, MoveType::Quiet);

        let bishop_moves_correct = move_list.0.contains(&desired_a7_move)
            && move_list.0.contains(&desired_b6_move)
            && move_list.0.contains(&desired_c5_move)
            && move_list.0.contains(&desired_g7_move)
            && move_list.0.contains(&desired_f6_move)
            && move_list.0.contains(&desired_e5_move)
            && move_list.0.contains(&desired_c3_move)
            && move_list.0.contains(&desired_b2_move)
            && move_list.0.contains(&desired_a1_move)
            && move_list.0.contains(&desired_e3_move)
            && move_list.0.len() == 10;

        assert!(bishop_moves_correct);
    }

    #[test]
    fn rook_moves() {
        let mut game = Game::initialise();
        let attack_tables = AttackTables::initialise();

        game.load_fen("3p4/8/8/8/3R1P2/8/8/8 w - - 0 1").unwrap();

        let source_square = Square::D4;

        let mut move_list = MoveList::new();

        move_list.generate_piece_moves(&game, Piece::Rook, source_square, &attack_tables);

        let desired_d8_move =
            Move::new(Square::D4, Square::D8, Piece::Rook, None, MoveType::Capture);
        let desired_d7_move = Move::new(Square::D4, Square::D7, Piece::Rook, None, MoveType::Quiet);
        let desired_d6_move = Move::new(Square::D4, Square::D6, Piece::Rook, None, MoveType::Quiet);
        let desired_d5_move = Move::new(Square::D4, Square::D5, Piece::Rook, None, MoveType::Quiet);

        let desired_a4_move = Move::new(Square::D4, Square::A4, Piece::Rook, None, MoveType::Quiet);
        let desired_b4_move = Move::new(Square::D4, Square::B4, Piece::Rook, None, MoveType::Quiet);
        let desired_c4_move = Move::new(Square::D4, Square::C4, Piece::Rook, None, MoveType::Quiet);

        let desired_e4_move = Move::new(Square::D4, Square::E4, Piece::Rook, None, MoveType::Quiet);

        let desired_d3_move = Move::new(Square::D4, Square::D3, Piece::Rook, None, MoveType::Quiet);
        let desired_d2_move = Move::new(Square::D4, Square::D2, Piece::Rook, None, MoveType::Quiet);
        let desired_d1_move = Move::new(Square::D4, Square::D1, Piece::Rook, None, MoveType::Quiet);

        let rook_moves_correct = move_list.0.contains(&desired_d8_move)
            && move_list.0.contains(&desired_d7_move)
            && move_list.0.contains(&desired_d6_move)
            && move_list.0.contains(&desired_d5_move)
            && move_list.0.contains(&desired_a4_move)
            && move_list.0.contains(&desired_b4_move)
            && move_list.0.contains(&desired_c4_move)
            && move_list.0.contains(&desired_e4_move)
            && move_list.0.contains(&desired_d3_move)
            && move_list.0.contains(&desired_d2_move)
            && move_list.0.contains(&desired_d1_move)
            && move_list.0.len() == 11;

        assert!(rook_moves_correct)
    }

    #[test]
    fn queen_moves() {
        let mut game = Game::initialise();
        let attack_tables = AttackTables::initialise();

        game.load_fen("3p4/6p1/8/8/3Q1P2/8/5P2/8 w - - 0 1")
            .unwrap();

        let source_square = Square::D4;

        let mut move_list = MoveList::new();

        move_list.generate_piece_moves(&game, Piece::Queen, source_square, &attack_tables);

        let desired_a7_move =
            Move::new(Square::D4, Square::A7, Piece::Queen, None, MoveType::Quiet);
        let desired_b6_move =
            Move::new(Square::D4, Square::B6, Piece::Queen, None, MoveType::Quiet);
        let desired_c5_move =
            Move::new(Square::D4, Square::C5, Piece::Queen, None, MoveType::Quiet);

        let desired_d8_move = Move::new(
            Square::D4,
            Square::D8,
            Piece::Queen,
            None,
            MoveType::Capture,
        );
        let desired_d7_move =
            Move::new(Square::D4, Square::D7, Piece::Queen, None, MoveType::Quiet);
        let desired_d6_move =
            Move::new(Square::D4, Square::D6, Piece::Queen, None, MoveType::Quiet);
        let desired_d5_move =
            Move::new(Square::D4, Square::D5, Piece::Queen, None, MoveType::Quiet);

        let desired_g7_move = Move::new(
            Square::D4,
            Square::G7,
            Piece::Queen,
            None,
            MoveType::Capture,
        );
        let desired_f6_move =
            Move::new(Square::D4, Square::F6, Piece::Queen, None, MoveType::Quiet);
        let desired_e5_move =
            Move::new(Square::D4, Square::E5, Piece::Queen, None, MoveType::Quiet);

        let desired_a4_move =
            Move::new(Square::D4, Square::A4, Piece::Queen, None, MoveType::Quiet);
        let desired_b4_move =
            Move::new(Square::D4, Square::B4, Piece::Queen, None, MoveType::Quiet);
        let desired_c4_move =
            Move::new(Square::D4, Square::C4, Piece::Queen, None, MoveType::Quiet);

        let desired_e4_move =
            Move::new(Square::D4, Square::E4, Piece::Queen, None, MoveType::Quiet);

        let desired_c3_move =
            Move::new(Square::D4, Square::C3, Piece::Queen, None, MoveType::Quiet);
        let desired_b2_move =
            Move::new(Square::D4, Square::B2, Piece::Queen, None, MoveType::Quiet);
        let desired_a1_move =
            Move::new(Square::D4, Square::A1, Piece::Queen, None, MoveType::Quiet);

        let desired_d3_move =
            Move::new(Square::D4, Square::D3, Piece::Queen, None, MoveType::Quiet);
        let desired_d2_move =
            Move::new(Square::D4, Square::D2, Piece::Queen, None, MoveType::Quiet);
        let desired_d1_move =
            Move::new(Square::D4, Square::D1, Piece::Queen, None, MoveType::Quiet);

        let desired_e3_move =
            Move::new(Square::D4, Square::E3, Piece::Queen, None, MoveType::Quiet);

        let queen_moves_correct = move_list.0.contains(&desired_a7_move)
            && move_list.0.contains(&desired_b6_move)
            && move_list.0.contains(&desired_c5_move)
            && move_list.0.contains(&desired_d8_move)
            && move_list.0.contains(&desired_d7_move)
            && move_list.0.contains(&desired_d6_move)
            && move_list.0.contains(&desired_d5_move)
            && move_list.0.contains(&desired_g7_move)
            && move_list.0.contains(&desired_f6_move)
            && move_list.0.contains(&desired_e5_move)
            && move_list.0.contains(&desired_a4_move)
            && move_list.0.contains(&desired_b4_move)
            && move_list.0.contains(&desired_c4_move)
            && move_list.0.contains(&desired_e4_move)
            && move_list.0.contains(&desired_c3_move)
            && move_list.0.contains(&desired_b2_move)
            && move_list.0.contains(&desired_a1_move)
            && move_list.0.contains(&desired_d3_move)
            && move_list.0.contains(&desired_d2_move)
            && move_list.0.contains(&desired_d1_move)
            && move_list.0.contains(&desired_e3_move)
            && move_list.0.len() == 21;

        assert!(queen_moves_correct);
    }

    #[test]
    fn king_moves() {
        let mut game = Game::initialise();
        let attack_tables = AttackTables::initialise();

        game.load_fen("8/8/8/2pP4/2PK4/2p5/8/8 w - - 0 1").unwrap();

        let source_square = Square::D4;

        let mut move_list = MoveList::new();

        move_list.generate_piece_moves(&game, Piece::King, source_square, &attack_tables);

        let desired_c5_move =
            Move::new(Square::D4, Square::C5, Piece::King, None, MoveType::Capture);
        let desired_e5_move = Move::new(Square::D4, Square::E5, Piece::King, None, MoveType::Quiet);
        let desired_e4_move = Move::new(Square::D4, Square::E4, Piece::King, None, MoveType::Quiet);
        let desired_c3_move =
            Move::new(Square::D4, Square::C3, Piece::King, None, MoveType::Capture);
        let desired_d3_move = Move::new(Square::D4, Square::D3, Piece::King, None, MoveType::Quiet);
        let desired_e3_move = Move::new(Square::D4, Square::E3, Piece::King, None, MoveType::Quiet);

        let king_moves_correct = move_list.0.contains(&desired_c5_move)
            && move_list.0.contains(&desired_e5_move)
            && move_list.0.contains(&desired_e4_move)
            && move_list.0.contains(&desired_c3_move)
            && move_list.0.contains(&desired_d3_move)
            && move_list.0.contains(&desired_e3_move)
            && move_list.0.len() == 6;

        assert!(king_moves_correct);
    }

    #[test]
    fn castling() {
        let mut game = Game::initialise();
        let attack_tables = AttackTables::initialise();

        game.load_fen("8/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();

        let mut move_list = MoveList::new();

        move_list.generate_castling_moves(&game, &attack_tables);

        let desired_short_castle = Move::new(
            Square::E1,
            Square::G1,
            Piece::King,
            None,
            MoveType::Castling,
        );
        let desired_long_castle = Move::new(
            Square::E1,
            Square::C1,
            Piece::King,
            None,
            MoveType::Castling,
        );

        let castling_moves_correct = move_list.0.contains(&desired_short_castle)
            && move_list.0.contains(&desired_long_castle)
            && move_list.0.len() == 2;

        assert!(castling_moves_correct);

        game.load_fen("8/8/8/8/8/5r2/8/R3K2R w KQ - 0 1").unwrap();

        let mut move_list = MoveList::new();

        move_list.generate_castling_moves(&game, &attack_tables);

        let desired_long_castle = Move::new(
            Square::E1,
            Square::C1,
            Piece::King,
            None,
            MoveType::Castling,
        );

        let castling_moves_correct =
            move_list.0.contains(&desired_long_castle) && move_list.0.len() == 1;

        assert!(castling_moves_correct);

        game.load_fen("8/8/8/8/8/5q2/8/R3K2R w KQ - 0 1").unwrap();

        let mut move_list = MoveList::new();

        move_list.generate_castling_moves(&game, &attack_tables);

        assert!(move_list.0.is_empty());

        game.load_fen("8/8/8/8/8/8/8/R3K2R w - - 0 1").unwrap();

        let mut move_list = MoveList::new();

        move_list.generate_castling_moves(&game, &attack_tables);

        assert!(move_list.0.is_empty());
    }

    #[test]
    fn parse_move() {
        let move_string = "e2e4";

        let move_search = MoveList::parse_move_string(move_string).unwrap();

        let desired_move_search = MoveSearch::new(Square::E2, Square::E4, None);

        assert_eq!(move_search, desired_move_search);

        let move_string = "e7e8q";

        let move_search = MoveList::parse_move_string(move_string).unwrap();

        let desired_move_search = MoveSearch::new(Square::E7, Square::E8, Some(Piece::Queen));

        assert_eq!(move_search, desired_move_search);

        let move_string = "e2e1r";

        let move_search = MoveList::parse_move_string(move_string).unwrap();

        let desired_move_search = MoveSearch::new(Square::E2, Square::E1, Some(Piece::Rook));

        assert_eq!(move_search, desired_move_search);

        let move_string = "d7d8b";

        let move_search = MoveList::parse_move_string(move_string).unwrap();

        let desired_move_search = MoveSearch::new(Square::D7, Square::D8, Some(Piece::Bishop));

        assert_eq!(move_search, desired_move_search);

        let move_string = "d2d1n";

        let move_search = MoveList::parse_move_string(move_string).unwrap();

        let desired_move_search = MoveSearch::new(Square::D2, Square::D1, Some(Piece::Knight));

        assert_eq!(move_search, desired_move_search);
    }
}
