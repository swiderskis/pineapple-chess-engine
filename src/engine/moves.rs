use super::{
    attack_tables::AttackTables,
    game::{Bitboard, CastlingType, Game, Piece, Side, Square},
    InputError,
};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use strum::IntoEnumIterator;

// Max legal moves from a position is currently thought to be 218
// https://www.chessprogramming.org/Encoding_Moves#MoveIndex
// 256 is given as a buffer due to pseudo-legal moves
pub const MAX_MOVE_LIST_SIZE: usize = 256;

const PROMOTION_PIECES: [Piece; 4] = [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen];

#[derive(Clone)]
pub struct MoveList {
    move_list: [Option<Move>; MAX_MOVE_LIST_SIZE],
    current_move_list_size: usize,
}

impl MoveList {
    pub fn new() -> Self {
        Self {
            move_list: [(); MAX_MOVE_LIST_SIZE].map(|_| None),
            current_move_list_size: 0,
        }
    }

    pub fn generate_moves(attack_tables: &AttackTables, game: &Game) -> Self {
        let mut move_list = Self::new();

        let side = game.side_to_move();

        for piece in Piece::iter() {
            let mut bitboard = game.piece_bitboard(piece, side);

            while let Some(source_square) = bitboard.get_lsb_square() {
                match piece {
                    Piece::Pawn => {
                        move_list.generate_pawn_moves(attack_tables, game, source_square)
                    }
                    _ => move_list.generate_piece_moves(attack_tables, game, piece, source_square),
                };

                bitboard.pop_bit(source_square);
            }
        }

        move_list
    }

    pub fn find_move(&self, move_search: MoveSearch) -> Result<Move, InputError> {
        for mv in self.move_list.iter().flatten() {
            if mv.source_square() == move_search.source_square
                && mv.target_square() == move_search.target_square
                && mv.promoted_piece() == move_search.promoted_piece
            {
                return Ok(mv.clone());
            }
        }

        Err(InputError::IllegalMove)
    }

    pub fn move_list(&self) -> &[Option<Move>; MAX_MOVE_LIST_SIZE] {
        &self.move_list
    }

    fn generate_pawn_moves(
        &mut self,
        attack_tables: &AttackTables,
        game: &Game,
        source_square: Square,
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
                self.push(Move::new(
                    source_square,
                    target_square,
                    Piece::Pawn,
                    Some(promoted_piece),
                    MoveType::Quiet,
                ));
            }
        } else if !game.is_square_occupied(target_square) {
            self.push(Move::new(
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
                    self.push(Move::new(
                        source_square,
                        target_square,
                        Piece::Pawn,
                        None,
                        MoveType::DoublePawnPush,
                    ));
                }
            }
        }

        let mut attacks = Self::generate_attacks(attack_tables, game, Piece::Pawn, source_square);

        while let Some(target_square) = attacks.get_lsb_square() {
            if pawn_ready_to_promote {
                for promoted_piece in PROMOTION_PIECES {
                    self.push(Move::new(
                        source_square,
                        target_square,
                        Piece::Pawn,
                        Some(promoted_piece),
                        MoveType::Capture,
                    ));
                }
            } else {
                self.push(Move::new(
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
                self.push(Move::new(
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
        attack_tables: &AttackTables,
        game: &Game,
        piece: Piece,
        source_square: Square,
    ) {
        let mut attacks = Self::generate_attacks(attack_tables, game, piece, source_square);

        while let Some(target_square) = attacks.get_lsb_square() {
            let move_type = if game.is_square_occupied(target_square) {
                MoveType::Capture
            } else {
                MoveType::Quiet
            };

            self.push(Move::new(
                source_square,
                target_square,
                piece,
                None,
                move_type,
            ));

            attacks.pop_bit(target_square);
        }

        if piece == Piece::King {
            self.generate_castling_moves(attack_tables, game);
        }
    }

    fn generate_castling_moves(&mut self, attack_tables: &AttackTables, game: &Game) {
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
                    self.push(Move::new(
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
                    self.push(Move::new(
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
                    self.push(Move::new(
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
                    self.push(Move::new(
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
        attack_tables: &AttackTables,
        game: &Game,
        piece: Piece,
        source_square: Square,
    ) -> Bitboard {
        let attack_table =
            attack_tables.attack_table(game.board(None), piece, game.side_to_move(), source_square);
        let valid_attack_squares = match piece {
            Piece::Pawn => game.board(Some(game.side_to_move().opponent_side())),
            _ => !game.board(Some(game.side_to_move())),
        };

        attack_table & valid_attack_squares
    }

    fn push(&mut self, mv: Move) {
        self.move_list[self.current_move_list_size] = Some(mv);
        self.current_move_list_size += 1;
    }

    pub fn _length(&self) -> u32 {
        let mut count = 0;

        for _ in self.move_list.iter().flatten() {
            count += 1;
        }

        count
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

#[derive(PartialEq)]
pub enum MoveFlag {
    All,
    Capture,
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

        let mut white_moves = MoveList::new();
        let mut black_moves = MoveList::new();

        white_moves.generate_pawn_moves(&attack_tables, &white_game, white_square);
        black_moves.generate_pawn_moves(&attack_tables, &black_game, black_square);

        let white_pawn_push = Move::new(Square::D3, Square::D4, Piece::Pawn, None, MoveType::Quiet);
        let black_pawn_push = Move::new(Square::D6, Square::D5, Piece::Pawn, None, MoveType::Quiet);

        let white_moves_correct = white_moves.move_list.contains(&Some(white_pawn_push))
            && white_moves.current_move_list_size == 1;
        let black_moves_correct = black_moves.move_list.contains(&Some(black_pawn_push))
            && black_moves.current_move_list_size == 1;

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

        let mut white_moves = MoveList::new();
        let mut black_moves = MoveList::new();

        white_moves.generate_pawn_moves(&attack_tables, &white_game, white_square);
        black_moves.generate_pawn_moves(&attack_tables, &black_game, black_square);

        let white_moves_correct = white_moves.current_move_list_size == 0;
        let black_moves_correct = black_moves.current_move_list_size == 0;

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

        let mut white_moves = MoveList::new();
        let mut black_moves = MoveList::new();

        white_moves.generate_pawn_moves(&attack_tables, &white_game, white_square);
        black_moves.generate_pawn_moves(&attack_tables, &black_game, black_square);

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

        let white_moves_correct = white_moves
            .move_list
            .contains(&Some(white_single_pawn_push))
            && white_moves
                .move_list
                .contains(&Some(white_double_pawn_push))
            && white_moves.current_move_list_size == 2;
        let black_moves_correct = black_moves
            .move_list
            .contains(&Some(black_single_pawn_push))
            && black_moves
                .move_list
                .contains(&Some(black_double_pawn_push))
            && black_moves.current_move_list_size == 2;

        assert!(white_moves_correct);
        assert!(black_moves_correct);

        // blocked from double push
        white_game
            .load_fen("8/8/8/8/3p4/8/3P4/8 w - - 0 1")
            .unwrap();
        black_game
            .load_fen("8/3p4/8/3P4/8/8/8/8 b - - 0 1")
            .unwrap();

        let mut white_moves = MoveList::new();
        let mut black_moves = MoveList::new();

        white_moves.generate_pawn_moves(&attack_tables, &white_game, white_square);
        black_moves.generate_pawn_moves(&attack_tables, &black_game, black_square);

        let white_single_pawn_push =
            Move::new(Square::D2, Square::D3, Piece::Pawn, None, MoveType::Quiet);
        let black_single_pawn_push =
            Move::new(Square::D7, Square::D6, Piece::Pawn, None, MoveType::Quiet);

        let white_moves_correct = white_moves
            .move_list
            .contains(&Some(white_single_pawn_push))
            && white_moves.current_move_list_size == 1;
        let black_moves_correct = black_moves
            .move_list
            .contains(&Some(black_single_pawn_push))
            && black_moves.current_move_list_size == 1;

        assert!(white_moves_correct);
        assert!(black_moves_correct);

        // blocked from single push
        white_game
            .load_fen("8/8/8/8/8/3p4/3P4/8 w - - 0 1")
            .unwrap();
        black_game
            .load_fen("8/3p4/3P4/8/8/8/8/8 b - - 0 1")
            .unwrap();

        let mut white_moves = MoveList::new();
        let mut black_moves = MoveList::new();

        white_moves.generate_pawn_moves(&attack_tables, &white_game, white_square);
        black_moves.generate_pawn_moves(&attack_tables, &black_game, black_square);

        let white_moves_correct = white_moves.current_move_list_size == 0;
        let black_moves_correct = black_moves.current_move_list_size == 0;

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

        let mut white_moves = MoveList::new();
        let mut black_moves = MoveList::new();

        white_moves.generate_pawn_moves(&attack_tables, &white_game, white_square);
        black_moves.generate_pawn_moves(&attack_tables, &black_game, black_square);

        let white_capture = Move::new(Square::D4, Square::E5, Piece::Pawn, None, MoveType::Capture);
        let black_capture = Move::new(Square::D5, Square::E4, Piece::Pawn, None, MoveType::Capture);

        assert!(white_moves.move_list.contains(&Some(white_capture)));
        assert!(black_moves.move_list.contains(&Some(black_capture)));
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

        let mut white_moves = MoveList::new();
        let mut black_moves = MoveList::new();

        white_moves.generate_pawn_moves(&attack_tables, &white_game, white_square);
        black_moves.generate_pawn_moves(&attack_tables, &black_game, black_square);

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

        let white_moves_correct = white_moves.move_list.contains(&Some(white_promotion_queen))
            && white_moves.move_list.contains(&Some(white_promotion_rook))
            && white_moves
                .move_list
                .contains(&Some(white_promotion_bishop))
            && white_moves
                .move_list
                .contains(&Some(white_promotion_knight))
            && white_moves.current_move_list_size == 4;
        let black_moves_correct = black_moves.move_list.contains(&Some(black_promotion_queen))
            && black_moves.move_list.contains(&Some(black_promotion_rook))
            && black_moves
                .move_list
                .contains(&Some(black_promotion_bishop))
            && black_moves
                .move_list
                .contains(&Some(black_promotion_knight))
            && black_moves.current_move_list_size == 4;

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

        let mut white_moves = MoveList::new();
        let mut black_moves = MoveList::new();

        white_moves.generate_pawn_moves(&attack_tables, &white_game, white_square);
        black_moves.generate_pawn_moves(&attack_tables, &black_game, black_square);

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

        let white_moves_correct = white_moves.move_list.contains(&Some(white_en_passant));
        let black_moves_correct = black_moves.move_list.contains(&Some(black_en_passant));

        assert!(white_moves_correct);
        assert!(black_moves_correct);
    }

    #[test]
    fn knight_moves() {
        let mut game = Game::initialise();
        let attack_tables = AttackTables::initialise();

        game.load_fen("8/8/2p5/5P2/3N4/1p6/2p1P3/8 w - - 0 1")
            .unwrap();

        let mut moves = MoveList::new();

        moves.generate_piece_moves(&attack_tables, &game, Piece::Knight, Square::D4);

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

        let knight_moves_correct = moves.move_list.contains(&Some(desired_c6_move))
            && moves.move_list.contains(&Some(desired_e6_move))
            && moves.move_list.contains(&Some(desired_b5_move))
            && moves.move_list.contains(&Some(desired_b3_move))
            && moves.move_list.contains(&Some(desired_f3_move))
            && moves.move_list.contains(&Some(desired_c2_move))
            && moves.current_move_list_size == 6;

        assert!(knight_moves_correct);
    }

    #[test]
    fn bishop_moves() {
        let mut game = Game::initialise();
        let attack_tables = AttackTables::initialise();

        game.load_fen("8/6p1/8/8/3B4/8/5P2/8 w - - 0 1").unwrap();

        let source_square = Square::D4;

        let mut moves = MoveList::new();

        moves.generate_piece_moves(&attack_tables, &game, Piece::Bishop, source_square);

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

        let bishop_moves_correct = moves.move_list.contains(&Some(desired_a7_move))
            && moves.move_list.contains(&Some(desired_b6_move))
            && moves.move_list.contains(&Some(desired_c5_move))
            && moves.move_list.contains(&Some(desired_g7_move))
            && moves.move_list.contains(&Some(desired_f6_move))
            && moves.move_list.contains(&Some(desired_e5_move))
            && moves.move_list.contains(&Some(desired_c3_move))
            && moves.move_list.contains(&Some(desired_b2_move))
            && moves.move_list.contains(&Some(desired_a1_move))
            && moves.move_list.contains(&Some(desired_e3_move))
            && moves.current_move_list_size == 10;

        assert!(bishop_moves_correct);
    }

    #[test]
    fn rook_moves() {
        let mut game = Game::initialise();
        let attack_tables = AttackTables::initialise();

        game.load_fen("3p4/8/8/8/3R1P2/8/8/8 w - - 0 1").unwrap();

        let source_square = Square::D4;

        let mut moves = MoveList::new();

        moves.generate_piece_moves(&attack_tables, &game, Piece::Rook, source_square);

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

        let rook_moves_correct = moves.move_list.contains(&Some(desired_d8_move))
            && moves.move_list.contains(&Some(desired_d7_move))
            && moves.move_list.contains(&Some(desired_d6_move))
            && moves.move_list.contains(&Some(desired_d5_move))
            && moves.move_list.contains(&Some(desired_a4_move))
            && moves.move_list.contains(&Some(desired_b4_move))
            && moves.move_list.contains(&Some(desired_c4_move))
            && moves.move_list.contains(&Some(desired_e4_move))
            && moves.move_list.contains(&Some(desired_d3_move))
            && moves.move_list.contains(&Some(desired_d2_move))
            && moves.move_list.contains(&Some(desired_d1_move))
            && moves.current_move_list_size == 11;

        assert!(rook_moves_correct)
    }

    #[test]
    fn queen_moves() {
        let mut game = Game::initialise();
        let attack_tables = AttackTables::initialise();

        game.load_fen("3p4/6p1/8/8/3Q1P2/8/5P2/8 w - - 0 1")
            .unwrap();

        let source_square = Square::D4;

        let mut moves = MoveList::new();

        moves.generate_piece_moves(&attack_tables, &game, Piece::Queen, source_square);

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

        let queen_moves_correct = moves.move_list.contains(&Some(desired_a7_move))
            && moves.move_list.contains(&Some(desired_b6_move))
            && moves.move_list.contains(&Some(desired_c5_move))
            && moves.move_list.contains(&Some(desired_d8_move))
            && moves.move_list.contains(&Some(desired_d7_move))
            && moves.move_list.contains(&Some(desired_d6_move))
            && moves.move_list.contains(&Some(desired_d5_move))
            && moves.move_list.contains(&Some(desired_g7_move))
            && moves.move_list.contains(&Some(desired_f6_move))
            && moves.move_list.contains(&Some(desired_e5_move))
            && moves.move_list.contains(&Some(desired_a4_move))
            && moves.move_list.contains(&Some(desired_b4_move))
            && moves.move_list.contains(&Some(desired_c4_move))
            && moves.move_list.contains(&Some(desired_e4_move))
            && moves.move_list.contains(&Some(desired_c3_move))
            && moves.move_list.contains(&Some(desired_b2_move))
            && moves.move_list.contains(&Some(desired_a1_move))
            && moves.move_list.contains(&Some(desired_d3_move))
            && moves.move_list.contains(&Some(desired_d2_move))
            && moves.move_list.contains(&Some(desired_d1_move))
            && moves.move_list.contains(&Some(desired_e3_move))
            && moves.current_move_list_size == 21;

        assert!(queen_moves_correct);
    }

    #[test]
    fn king_moves() {
        let mut game = Game::initialise();
        let attack_tables = AttackTables::initialise();

        game.load_fen("8/8/8/2pP4/2PK4/2p5/8/8 w - - 0 1").unwrap();

        let source_square = Square::D4;

        let mut moves = MoveList::new();

        moves.generate_piece_moves(&attack_tables, &game, Piece::King, source_square);

        let desired_c5_move =
            Move::new(Square::D4, Square::C5, Piece::King, None, MoveType::Capture);
        let desired_e5_move = Move::new(Square::D4, Square::E5, Piece::King, None, MoveType::Quiet);
        let desired_e4_move = Move::new(Square::D4, Square::E4, Piece::King, None, MoveType::Quiet);
        let desired_c3_move =
            Move::new(Square::D4, Square::C3, Piece::King, None, MoveType::Capture);
        let desired_d3_move = Move::new(Square::D4, Square::D3, Piece::King, None, MoveType::Quiet);
        let desired_e3_move = Move::new(Square::D4, Square::E3, Piece::King, None, MoveType::Quiet);

        let king_moves_correct = moves.move_list.contains(&Some(desired_c5_move))
            && moves.move_list.contains(&Some(desired_e5_move))
            && moves.move_list.contains(&Some(desired_e4_move))
            && moves.move_list.contains(&Some(desired_c3_move))
            && moves.move_list.contains(&Some(desired_d3_move))
            && moves.move_list.contains(&Some(desired_e3_move))
            && moves.current_move_list_size == 6;

        assert!(king_moves_correct);
    }

    #[test]
    fn castling() {
        let mut game = Game::initialise();
        let attack_tables = AttackTables::initialise();

        game.load_fen("8/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();

        let mut moves = MoveList::new();

        moves.generate_castling_moves(&attack_tables, &game);

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

        let castling_moves_correct = moves.move_list.contains(&Some(desired_short_castle))
            && moves.move_list.contains(&Some(desired_long_castle))
            && moves.current_move_list_size == 2;

        assert!(castling_moves_correct);

        game.load_fen("8/8/8/8/8/5r2/8/R3K2R w KQ - 0 1").unwrap();

        let mut moves = MoveList::new();

        moves.generate_castling_moves(&attack_tables, &game);

        let desired_long_castle = Move::new(
            Square::E1,
            Square::C1,
            Piece::King,
            None,
            MoveType::Castling,
        );

        let castling_moves_correct = moves.move_list.contains(&Some(desired_long_castle))
            && moves.current_move_list_size == 1;

        assert!(castling_moves_correct);

        game.load_fen("8/8/8/8/8/5q2/8/R3K2R w KQ - 0 1").unwrap();

        let mut moves = MoveList::new();

        moves.generate_castling_moves(&attack_tables, &game);

        assert!(moves.current_move_list_size == 0);

        game.load_fen("8/8/8/8/8/8/8/R3K2R w - - 0 1").unwrap();

        let mut moves = MoveList::new();

        moves.generate_castling_moves(&attack_tables, &game);

        assert!(moves.current_move_list_size == 0);
    }
}
