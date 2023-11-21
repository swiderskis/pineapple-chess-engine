use super::{
    attack_tables,
    game::{CastlingType, Game},
    Bitboard, Piece, Side, Square,
};
use num_derive::{FromPrimitive, ToPrimitive};
#[allow(unused_imports)]
use num_traits::{FromPrimitive, ToPrimitive};

#[derive(Clone, Copy, Debug, FromPrimitive, PartialEq, ToPrimitive)]
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

#[derive(Debug, PartialEq)]
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

    pub fn capture(&self) -> bool {
        self.move_type == MoveType::Capture
    }

    pub fn double_pawn_push(&self) -> bool {
        self.move_type == MoveType::DoublePawnPush
    }

    pub fn _en_passant(&self) -> bool {
        self.move_type == MoveType::EnPassant
    }

    pub fn _castling(&self) -> bool {
        self.move_type == MoveType::Castling
    }

    pub fn _print_move(&self) {
        print!(
            "{}{}",
            self.source_square()._to_lowercase_string(),
            self.target_square()._to_lowercase_string(),
        );
        if let Some(promoted_piece) = self.promoted_piece() {
            print!("{}", promoted_piece._to_char(Side::Black))
        }
        println!();
    }
}

pub fn generate_moves(game: &Game) -> Vec<Move> {
    let mut move_list = Vec::new();

    let side = game.side_to_move();

    for (mut bitboard, piece) in game.side_bitboards(side) {
        while let Some(source_square_index) = bitboard.get_lsb_index() {
            let source_square = Square::from_usize(source_square_index).unwrap();

            let attacks = generate_attacks(game, piece, source_square);

            let generated_moves = match piece {
                Piece::Pawn => {
                    let attack_table = attack_tables::ATTACK_TABLES.attack_table(
                        game.board(None),
                        piece,
                        side,
                        source_square,
                    );

                    generate_pawn_moves(attack_table, attacks, game, source_square_index)
                }
                _ => generate_piece_moves(attacks, game, piece, source_square),
            };

            move_list.extend(generated_moves);

            bitboard.pop_bit(Square::from_usize(source_square_index).unwrap());
        }
    }

    move_list
}

fn generate_pawn_moves(
    attack_table: Bitboard,
    mut attacks: Bitboard,
    game: &Game,
    source_square_index: usize,
) -> Vec<Move> {
    let promotion_pieces = [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight];

    let mut move_list = Vec::new();

    let second_rank = Bitboard::new(0xFF_0000_0000_0000);
    let seventh_rank = Bitboard::new(0xFF00);

    let side = game.side_to_move();

    let source_square = Square::from_usize(source_square_index).unwrap();
    let target_square = match side {
        Side::White => Square::from_usize(source_square_index - 8),
        Side::Black => Square::from_usize(source_square_index + 8),
    }
    .unwrap();
    let target_square_occupied = game.is_square_occupied(target_square);

    let single_piece = Bitboard::from_square(source_square);

    let piece_on_second_rank = second_rank & single_piece != 0u64;
    let piece_on_seventh_rank = seventh_rank & single_piece != 0u64;

    if ((side == Side::White && piece_on_seventh_rank)
        || (side == Side::Black && piece_on_second_rank))
        && !target_square_occupied
    {
        for promoted_piece in promotion_pieces {
            move_list.push(Move::new(
                source_square,
                target_square,
                Piece::Pawn,
                Some(promoted_piece),
                MoveType::Quiet,
            ));
        }
    } else if !target_square_occupied {
        move_list.push(Move::new(
            source_square,
            target_square,
            Piece::Pawn,
            None,
            MoveType::Quiet,
        ));

        let double_push_target_square = if side == Side::White && piece_on_second_rank {
            Some(Square::from_usize(source_square_index - 16).unwrap())
        } else if side == Side::Black && piece_on_seventh_rank {
            Some(Square::from_usize(source_square_index + 16).unwrap())
        } else {
            None
        };

        if let Some(target_square) = double_push_target_square {
            let target_square_occupied = game.is_square_occupied(target_square);

            if !target_square_occupied {
                move_list.push(Move::new(
                    source_square,
                    target_square,
                    Piece::Pawn,
                    None,
                    MoveType::DoublePawnPush,
                ));
            }
        }
    }

    while let Some(target_square_index) = attacks.get_lsb_index() {
        let target_square = Square::from_usize(target_square_index).unwrap();

        if (side == Side::White && piece_on_seventh_rank)
            || (side == Side::Black && piece_on_second_rank)
        {
            for promoted_piece in promotion_pieces {
                move_list.push(Move::new(
                    source_square,
                    target_square,
                    Piece::Pawn,
                    Some(promoted_piece),
                    MoveType::Capture,
                ));
            }
        } else {
            move_list.push(Move::new(
                source_square,
                target_square,
                Piece::Pawn,
                None,
                MoveType::Capture,
            ));
        }

        attacks.pop_bit(target_square);
    }

    if let Some(target_square) = game.en_passant_square() {
        let en_passant_square_attacked =
            attack_table & Bitboard::from_square(target_square) != 0u64;

        if en_passant_square_attacked {
            move_list.push(Move::new(
                source_square,
                target_square,
                Piece::Pawn,
                None,
                MoveType::EnPassant,
            ));
        }
    }

    move_list
}

fn generate_piece_moves(
    mut attacks: Bitboard,
    game: &Game,
    piece: Piece,
    source_square: Square,
) -> Vec<Move> {
    let mut move_list = Vec::new();

    while let Some(target_square_index) = attacks.get_lsb_index() {
        let target_square = Square::from_usize(target_square_index).unwrap();

        let move_type = if game.is_square_occupied(target_square) {
            MoveType::Capture
        } else {
            MoveType::Quiet
        };

        move_list.push(Move::new(
            source_square,
            target_square,
            piece,
            None,
            move_type,
        ));

        attacks.pop_bit(target_square);
    }

    if piece == Piece::King {
        move_list.extend(generate_castling_moves(game));
    }

    move_list
}

fn generate_castling_moves(game: &Game) -> Vec<Move> {
    let mut move_list = Vec::new();

    let side = game.side_to_move();
    let opponent_side = side.opponent_side();

    let (b_file_square, c_file_square, d_file_square, e_file_square, f_file_square, g_file_square) =
        match side {
            Side::White => (
                Square::B1,
                Square::C1,
                Square::D1,
                Square::E1,
                Square::F1,
                Square::G1,
            ),
            Side::Black => (
                Square::B8,
                Square::C8,
                Square::D8,
                Square::E8,
                Square::F8,
                Square::G8,
            ),
        };
    let (short_castle, long_castle) = match side {
        Side::White => (CastlingType::WhiteShort, CastlingType::WhiteLong),
        Side::Black => (CastlingType::BlackShort, CastlingType::BlackLong),
    };

    if game.castling_type_allowed(short_castle)
        && !game.is_square_occupied(f_file_square)
        && !game.is_square_occupied(g_file_square)
        && !game.is_square_attacked(opponent_side, e_file_square)
        && !game.is_square_attacked(opponent_side, f_file_square)
    {
        move_list.push(Move::new(
            e_file_square,
            g_file_square,
            Piece::King,
            None,
            MoveType::Castling,
        ));
    }

    if game.castling_type_allowed(long_castle)
        && !game.is_square_occupied(b_file_square)
        && !game.is_square_occupied(c_file_square)
        && !game.is_square_occupied(d_file_square)
        && !game.is_square_attacked(opponent_side, d_file_square)
        && !game.is_square_attacked(opponent_side, e_file_square)
    {
        move_list.push(Move::new(
            e_file_square,
            c_file_square,
            Piece::King,
            None,
            MoveType::Castling,
        ));
    }

    move_list
}

fn generate_attacks(game: &Game, piece: Piece, source_square: Square) -> Bitboard {
    let attack_table = attack_tables::ATTACK_TABLES.attack_table(
        game.board(None),
        piece,
        game.side_to_move(),
        source_square,
    );
    let valid_attack_squares = match piece {
        Piece::Pawn => game.board(Some(game.side_to_move().opponent_side())),
        _ => !game.board(Some(game.side_to_move())),
    };

    attack_table & valid_attack_squares
}

#[cfg(test)]
#[allow(clippy::unusual_byte_groupings)]
mod tests {
    use super::*;

    #[test]
    fn single_pawn_push() {
        let white_game = Game::initialise("8/8/8/8/8/3P4/8/8 w - - 0 1");
        let black_game = Game::initialise("8/8/3p4/8/8/8/8/8 b - - 0 1");

        let white_square = Square::D3;
        let black_square = Square::D6;

        let white_attack_table = attack_tables::ATTACK_TABLES.attack_table(
            white_game.board(None),
            Piece::Pawn,
            Side::White,
            white_square,
        );
        let black_attack_table = attack_tables::ATTACK_TABLES.attack_table(
            black_game.board(None),
            Piece::Pawn,
            Side::Black,
            black_square,
        );

        let white_attacks = generate_attacks(&white_game, Piece::Pawn, white_square);
        let black_attacks = generate_attacks(&black_game, Piece::Pawn, black_square);

        let white_moves = generate_pawn_moves(
            white_attack_table,
            white_attacks,
            &white_game,
            white_square.to_usize().unwrap(),
        );
        let black_moves = generate_pawn_moves(
            black_attack_table,
            black_attacks,
            &black_game,
            black_square.to_usize().unwrap(),
        );

        let white_pawn_push = Move::new(Square::D3, Square::D4, Piece::Pawn, None, MoveType::Quiet);
        let black_pawn_push = Move::new(Square::D6, Square::D5, Piece::Pawn, None, MoveType::Quiet);

        let white_moves_correct = white_moves.contains(&white_pawn_push) && white_moves.len() == 1;
        let black_moves_correct = black_moves.contains(&black_pawn_push) && black_moves.len() == 1;

        assert!(white_moves_correct);
        assert!(black_moves_correct);

        // blocked from pushing
        let white_game = Game::initialise("8/8/8/8/3p4/3P4/8/8 w - - 0 1");
        let black_game = Game::initialise("8/8/3p4/3P4/8/8/8/8 b - - 0 1");

        let white_square = Square::D3;
        let black_square = Square::D6;

        let white_attack_table = attack_tables::ATTACK_TABLES.attack_table(
            white_game.board(None),
            Piece::Pawn,
            Side::White,
            white_square,
        );
        let black_attack_table = attack_tables::ATTACK_TABLES.attack_table(
            black_game.board(None),
            Piece::Pawn,
            Side::Black,
            black_square,
        );

        let white_attacks = generate_attacks(&white_game, Piece::Pawn, white_square);
        let black_attacks = generate_attacks(&black_game, Piece::Pawn, black_square);

        let white_moves = generate_pawn_moves(
            white_attack_table,
            white_attacks,
            &white_game,
            white_square.to_usize().unwrap(),
        );
        let black_moves = generate_pawn_moves(
            black_attack_table,
            black_attacks,
            &black_game,
            black_square.to_usize().unwrap(),
        );

        let white_moves_correct = white_moves.is_empty();
        let black_moves_correct = black_moves.is_empty();

        assert!(white_moves_correct);
        assert!(black_moves_correct);
    }

    #[test]
    fn double_pawn_push() {
        let white_game = Game::initialise("8/8/8/8/8/8/3P4/8 w - - 0 1");
        let black_game = Game::initialise("8/3p4/8/8/8/8/8/8 b - - 0 1");

        let white_square = Square::D2;
        let black_square = Square::D7;

        let white_attack_table = attack_tables::ATTACK_TABLES.attack_table(
            white_game.board(None),
            Piece::Pawn,
            Side::White,
            white_square,
        );
        let black_attack_table = attack_tables::ATTACK_TABLES.attack_table(
            black_game.board(None),
            Piece::Pawn,
            Side::Black,
            black_square,
        );

        let white_attacks = generate_attacks(&white_game, Piece::Pawn, white_square);
        let black_attacks = generate_attacks(&black_game, Piece::Pawn, black_square);

        let white_moves = generate_pawn_moves(
            white_attack_table,
            white_attacks,
            &white_game,
            white_square.to_usize().unwrap(),
        );
        let black_moves = generate_pawn_moves(
            black_attack_table,
            black_attacks,
            &black_game,
            black_square.to_usize().unwrap(),
        );

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

        let white_moves_correct = white_moves.contains(&white_single_pawn_push)
            && white_moves.contains(&white_double_pawn_push)
            && white_moves.len() == 2;
        let black_moves_correct = black_moves.contains(&black_single_pawn_push)
            && black_moves.contains(&black_double_pawn_push)
            && black_moves.len() == 2;

        assert!(white_moves_correct);
        assert!(black_moves_correct);

        // blocked from double push
        let white_game = Game::initialise("8/8/8/8/3p4/8/3P4/8 w - - 0 1");
        let black_game = Game::initialise("8/3p4/8/3P4/8/8/8/8 b - - 0 1");

        let white_moves = generate_pawn_moves(
            white_attack_table,
            white_attacks,
            &white_game,
            white_square.to_usize().unwrap(),
        );
        let black_moves = generate_pawn_moves(
            black_attack_table,
            black_attacks,
            &black_game,
            black_square.to_usize().unwrap(),
        );

        let white_moves_correct =
            white_moves.contains(&white_single_pawn_push) && white_moves.len() == 1;
        let black_moves_correct =
            black_moves.contains(&black_single_pawn_push) && black_moves.len() == 1;

        assert!(white_moves_correct);
        assert!(black_moves_correct);

        // blocked from single push
        let white_game = Game::initialise("8/8/8/8/8/3p4/3P4/8 w - - 0 1");
        let black_game = Game::initialise("8/3p4/3P4/8/8/8/8/8 b - - 0 1");

        let white_moves = generate_pawn_moves(
            white_attack_table,
            white_attacks,
            &white_game,
            white_square.to_usize().unwrap(),
        );
        let black_moves = generate_pawn_moves(
            black_attack_table,
            black_attacks,
            &black_game,
            black_square.to_usize().unwrap(),
        );

        let white_moves_correct = white_moves.is_empty();
        let black_moves_correct = black_moves.is_empty();

        assert!(white_moves_correct);
        assert!(black_moves_correct);
    }

    #[test]
    fn pawn_capture() {
        let white_game = Game::initialise("8/8/8/2P1p3/3P4/8/8/8 w - - 0 1");
        let black_game = Game::initialise("8/8/8/3p4/2p1P3/8/8/8 b - - 0 1");

        let white_square = Square::D4;
        let black_square = Square::D5;

        let white_attack_table = attack_tables::ATTACK_TABLES.attack_table(
            white_game.board(None),
            Piece::Pawn,
            Side::White,
            white_square,
        );
        let black_attack_table = attack_tables::ATTACK_TABLES.attack_table(
            black_game.board(None),
            Piece::Pawn,
            Side::Black,
            black_square,
        );

        let white_attacks = generate_attacks(&white_game, Piece::Pawn, white_square);
        let black_attacks = generate_attacks(&black_game, Piece::Pawn, black_square);

        let white_moves = generate_pawn_moves(
            white_attack_table,
            white_attacks,
            &white_game,
            white_square.to_usize().unwrap(),
        );
        let black_moves = generate_pawn_moves(
            black_attack_table,
            black_attacks,
            &black_game,
            black_square.to_usize().unwrap(),
        );

        let white_capture = Move::new(Square::D4, Square::E5, Piece::Pawn, None, MoveType::Capture);
        let black_capture = Move::new(Square::D5, Square::E4, Piece::Pawn, None, MoveType::Capture);

        assert!(white_moves.contains(&white_capture));
        assert!(black_moves.contains(&black_capture));
    }

    #[test]
    fn pawn_promotion() {
        let white_game = Game::initialise("8/3P4/8/8/8/8/8/8 w - - 0 1");
        let black_game = Game::initialise("8/8/8/8/8/8/3p4/8 b - - 0 1");

        let white_square = Square::D7;
        let black_square = Square::D2;

        let white_attack_table = attack_tables::ATTACK_TABLES.attack_table(
            white_game.board(None),
            Piece::Pawn,
            Side::White,
            white_square,
        );
        let black_attack_table = attack_tables::ATTACK_TABLES.attack_table(
            black_game.board(None),
            Piece::Pawn,
            Side::Black,
            black_square,
        );

        let white_attacks = generate_attacks(&white_game, Piece::Pawn, white_square);
        let black_attacks = generate_attacks(&black_game, Piece::Pawn, black_square);

        let white_moves = generate_pawn_moves(
            white_attack_table,
            white_attacks,
            &white_game,
            white_square.to_usize().unwrap(),
        );
        let black_moves = generate_pawn_moves(
            black_attack_table,
            black_attacks,
            &black_game,
            black_square.to_usize().unwrap(),
        );

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

        let white_moves_correct = white_moves.contains(&white_promotion_queen)
            && white_moves.contains(&white_promotion_rook)
            && white_moves.contains(&white_promotion_bishop)
            && white_moves.contains(&white_promotion_knight)
            && white_moves.len() == 4;
        let black_moves_correct = black_moves.contains(&black_promotion_queen)
            && black_moves.contains(&black_promotion_rook)
            && black_moves.contains(&black_promotion_bishop)
            && black_moves.contains(&black_promotion_knight)
            && black_moves.len() == 4;

        assert!(white_moves_correct);
        assert!(black_moves_correct);
    }

    #[test]
    fn en_passant() {
        let white_game = Game::initialise("8/8/8/3Pp3/8/8/8/8 w - e6 0 1");
        let black_game = Game::initialise("8/8/8/8/3pP3/8/8/8 b - e3 0 1");

        let white_square = Square::D5;
        let black_square = Square::D4;

        let white_attack_table = attack_tables::ATTACK_TABLES.attack_table(
            white_game.board(None),
            Piece::Pawn,
            Side::White,
            white_square,
        );
        let black_attack_table = attack_tables::ATTACK_TABLES.attack_table(
            black_game.board(None),
            Piece::Pawn,
            Side::Black,
            black_square,
        );

        let white_attacks = generate_attacks(&white_game, Piece::Pawn, white_square);
        let black_attacks = generate_attacks(&black_game, Piece::Pawn, black_square);

        let white_moves = generate_pawn_moves(
            white_attack_table,
            white_attacks,
            &white_game,
            white_square.to_usize().unwrap(),
        );
        let black_moves = generate_pawn_moves(
            black_attack_table,
            black_attacks,
            &black_game,
            black_square.to_usize().unwrap(),
        );

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

        let white_moves_correct = white_moves.contains(&white_en_passant);
        let black_moves_correct = black_moves.contains(&black_en_passant);

        assert!(white_moves_correct);
        assert!(black_moves_correct);
    }

    #[test]
    fn knight_moves() {
        let game = Game::initialise("8/8/2p5/5P2/3N4/1p6/2p1P3/8 w - - 0 1");

        let source_square = Square::D4;

        let attacks = generate_attacks(&game, Piece::Knight, source_square);

        let moves = generate_piece_moves(attacks, &game, Piece::Knight, Square::D4);

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

        let knight_moves_correct = moves.contains(&desired_c6_move)
            && moves.contains(&desired_e6_move)
            && moves.contains(&desired_b5_move)
            && moves.contains(&desired_b3_move)
            && moves.contains(&desired_f3_move)
            && moves.contains(&desired_c2_move)
            && moves.len() == 6;

        assert!(knight_moves_correct);
    }

    #[test]
    fn bishop_moves() {
        let game = Game::initialise("8/6p1/8/8/3B4/8/5P2/8 w - - 0 1");

        let source_square = Square::D4;

        let attacks = generate_attacks(&game, Piece::Bishop, source_square);

        let moves = generate_piece_moves(attacks, &game, Piece::Bishop, Square::D4);

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

        let bishop_moves_correct = moves.contains(&desired_a7_move)
            && moves.contains(&desired_b6_move)
            && moves.contains(&desired_c5_move)
            && moves.contains(&desired_g7_move)
            && moves.contains(&desired_f6_move)
            && moves.contains(&desired_e5_move)
            && moves.contains(&desired_c3_move)
            && moves.contains(&desired_b2_move)
            && moves.contains(&desired_a1_move)
            && moves.contains(&desired_e3_move)
            && moves.len() == 10;

        assert!(bishop_moves_correct);
    }

    #[test]
    fn rook_moves() {
        let game = Game::initialise("3p4/8/8/8/3R1P2/8/8/8 w - - 0 1");

        let source_square = Square::D4;

        let attacks = generate_attacks(&game, Piece::Rook, source_square);

        let moves = generate_piece_moves(attacks, &game, Piece::Rook, Square::D4);

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

        let rook_moves_correct = moves.contains(&desired_d8_move)
            && moves.contains(&desired_d7_move)
            && moves.contains(&desired_d6_move)
            && moves.contains(&desired_d5_move)
            && moves.contains(&desired_a4_move)
            && moves.contains(&desired_b4_move)
            && moves.contains(&desired_c4_move)
            && moves.contains(&desired_e4_move)
            && moves.contains(&desired_d3_move)
            && moves.contains(&desired_d2_move)
            && moves.contains(&desired_d1_move)
            && moves.len() == 11;

        assert!(rook_moves_correct)
    }

    #[test]
    fn queen_moves() {
        let game = Game::initialise("3p4/6p1/8/8/3Q1P2/8/5P2/8 w - - 0 1");

        let source_square = Square::D4;

        let attacks = generate_attacks(&game, Piece::Queen, source_square);

        let moves = generate_piece_moves(attacks, &game, Piece::Queen, Square::D4);

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

        let queen_moves_correct = moves.contains(&desired_a7_move)
            && moves.contains(&desired_b6_move)
            && moves.contains(&desired_c5_move)
            && moves.contains(&desired_d8_move)
            && moves.contains(&desired_d7_move)
            && moves.contains(&desired_d6_move)
            && moves.contains(&desired_d5_move)
            && moves.contains(&desired_g7_move)
            && moves.contains(&desired_f6_move)
            && moves.contains(&desired_e5_move)
            && moves.contains(&desired_a4_move)
            && moves.contains(&desired_b4_move)
            && moves.contains(&desired_c4_move)
            && moves.contains(&desired_e4_move)
            && moves.contains(&desired_c3_move)
            && moves.contains(&desired_b2_move)
            && moves.contains(&desired_a1_move)
            && moves.contains(&desired_d3_move)
            && moves.contains(&desired_d2_move)
            && moves.contains(&desired_d1_move)
            && moves.contains(&desired_e3_move)
            && moves.len() == 21;

        assert!(queen_moves_correct);
    }

    #[test]
    fn king_moves() {
        let game = Game::initialise("8/8/8/2pP4/2PK4/2p5/8/8 w - - 0 1");

        let source_square = Square::D4;

        let attacks = generate_attacks(&game, Piece::King, source_square);

        let moves = generate_piece_moves(attacks, &game, Piece::King, Square::D4);

        let desired_c5_move =
            Move::new(Square::D4, Square::C5, Piece::King, None, MoveType::Capture);
        let desired_e5_move = Move::new(Square::D4, Square::E5, Piece::King, None, MoveType::Quiet);
        let desired_e4_move = Move::new(Square::D4, Square::E4, Piece::King, None, MoveType::Quiet);
        let desired_c3_move =
            Move::new(Square::D4, Square::C3, Piece::King, None, MoveType::Capture);
        let desired_d3_move = Move::new(Square::D4, Square::D3, Piece::King, None, MoveType::Quiet);
        let desired_e3_move = Move::new(Square::D4, Square::E3, Piece::King, None, MoveType::Quiet);

        let king_moves_correct = moves.contains(&desired_c5_move)
            && moves.contains(&desired_e5_move)
            && moves.contains(&desired_e4_move)
            && moves.contains(&desired_c3_move)
            && moves.contains(&desired_d3_move)
            && moves.contains(&desired_e3_move)
            && moves.len() == 6;

        assert!(king_moves_correct);
    }

    #[test]
    fn castling() {
        let game = Game::initialise("8/8/8/8/8/8/8/R3K2R w KQ - 0 1");

        let moves = generate_castling_moves(&game);

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

        let castling_moves_correct = moves.contains(&desired_short_castle)
            && moves.contains(&desired_long_castle)
            && moves.len() == 2;

        assert!(castling_moves_correct);

        let game = Game::initialise("8/8/8/8/8/5r2/8/R3K2R w KQ - 0 1");

        let moves = generate_castling_moves(&game);

        let castling_moves_correct = moves.contains(&desired_long_castle) && moves.len() == 1;

        assert!(castling_moves_correct);

        let game = Game::initialise("8/8/8/8/8/5q2/8/R3K2R w KQ - 0 1");

        let moves = generate_castling_moves(&game);

        assert!(moves.is_empty());

        let game = Game::initialise("8/8/8/8/8/8/8/R3K2R w - - 0 1");

        let moves = generate_castling_moves(&game);

        assert!(moves.is_empty());
    }
}
