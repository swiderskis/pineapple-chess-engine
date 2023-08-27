use super::{
    attack_tables::AttackTables,
    game::{CastlingType, Game},
    Bitboard, EnumToInt, IntToEnum, Piece, Side, Square,
};
use num_derive::{FromPrimitive, ToPrimitive};

static BLACK_PIECE_OFFSET: u32 = 6;
static NO_PIECE_VALUE: u32 = 0b1111;

#[derive(FromPrimitive, ToPrimitive)]
pub enum MoveType {
    Quiet = 0b0000,
    Capture = 0b0001,
    DoublePawnPush = 0b0010,
    EnPassant = 0b0101,
    Castling = 0b1000,
}

impl EnumToInt for MoveType {}
impl IntToEnum for MoveType {}

#[derive(PartialEq)]
pub enum MoveFlag {
    All,
    Capture,
}

pub struct MoveList {
    moves: Vec<Move>,
}

impl MoveList {
    pub fn new() -> Self {
        MoveList { moves: Vec::new() }
    }

    pub fn print_move(&self, index: usize) {
        print!(
            "{}{}",
            self.moves[index].source_square().to_lowercase_string(),
            self.moves[index].target_square().to_lowercase_string(),
        );
        if let Some(promoted_piece) = self.moves[index].promoted_piece() {
            print!("{}", promoted_piece.to_char(None))
        }
        println!();
    }

    pub fn print_move_list(&self) {
        self.moves.iter().for_each(|mv| {
            print!(
                "Move: {}{}",
                mv.source_square().to_lowercase_string(),
                mv.target_square().to_lowercase_string(),
            );
            if let Some(promoted_piece) = mv.promoted_piece() {
                print!("{}", promoted_piece.to_char(None))
            }
            println!();
            print!("Piece: {} | ", mv.piece());
            print!("Capture: {} | ", mv.capture());
            print!("Double push: {} | ", mv.double_pawn_push());
            print!("En passant: {} | ", mv.en_passant());
            println!("Castling: {}", mv.castling());
            println!("---");
        });
    }

    pub fn moves(&self) -> &Vec<Move> {
        &self.moves
    }

    fn add_move(
        &mut self,
        source_square: &Square,
        target_square: &Square,
        piece: &Piece,
        side: &Side,
        promoted_piece: Option<&Piece>,
        move_type: MoveType,
    ) {
        let mv = Move::new(
            source_square,
            target_square,
            piece,
            side,
            promoted_piece,
            move_type,
        );
        self.push_move(mv);
    }

    fn push_move(&mut self, mv: Move) {
        self.moves.push(mv);
    }

    fn append_moves(&mut self, move_list: &mut MoveList) {
        self.moves.append(&mut move_list.moves);
    }
}

#[derive(PartialEq)]
pub struct Move {
    move_information: u32,
}

impl Move {
    fn new(
        source_square: &Square,
        target_square: &Square,
        piece: &Piece,
        side: &Side,
        promoted_piece: Option<&Piece>,
        move_type: MoveType,
    ) -> Self {
        let side_value_offset = match side {
            Side::White => 0,
            Side::Black => BLACK_PIECE_OFFSET,
        };
        let piece_value = piece.as_u32() + side_value_offset;
        let promoted_piece_value = if let Some(promoted_piece) = promoted_piece {
            match promoted_piece {
                Piece::Pawn | Piece::King => {
                    eprintln!("Attempted to promote pawn to pawn or king");
                    Piece::Queen.as_u32() + side_value_offset
                }
                _ => promoted_piece.as_u32() + side_value_offset,
            }
        } else {
            NO_PIECE_VALUE
        };
        let move_flags = move_type.as_u32();

        let mut move_information = source_square.as_u32();
        move_information |= target_square.as_u32() << 6;
        move_information |= piece_value << 12;
        move_information |= promoted_piece_value << 16;
        move_information |= move_flags << 20;

        Move { move_information }
    }

    pub fn source_square(&self) -> Square {
        let source_square_index = self.move_information & 0x3F;

        Square::new_from_index(source_square_index as usize)
    }

    pub fn target_square(&self) -> Square {
        let target_square_index = (self.move_information & 0xFC0) >> 6;

        Square::new_from_index(target_square_index as usize)
    }

    pub fn piece(&self) -> Piece {
        let piece_value = (self.move_information & 0xF000) >> 12;
        let piece_value = if piece_value >= BLACK_PIECE_OFFSET {
            piece_value - BLACK_PIECE_OFFSET
        } else {
            piece_value
        };

        Piece::new_from_u32(piece_value)
    }

    pub fn promoted_piece(&self) -> Option<Piece> {
        let promoted_piece_value = (self.move_information & 0xF0000) >> 16;

        if promoted_piece_value == NO_PIECE_VALUE {
            return None;
        }

        let promoted_piece_value = if promoted_piece_value >= BLACK_PIECE_OFFSET {
            promoted_piece_value - BLACK_PIECE_OFFSET
        } else {
            promoted_piece_value
        };

        Some(Piece::new_from_u32(promoted_piece_value))
    }

    pub fn move_type(&self) -> MoveType {
        let move_type_index = (self.move_information & 0xF00000) >> 20;

        MoveType::new_from_u32(move_type_index)
    }

    pub fn capture(&self) -> bool {
        self.move_information & 0x100000 != 0
    }

    pub fn double_pawn_push(&self) -> bool {
        self.move_information & 0x200000 != 0
    }

    pub fn en_passant(&self) -> bool {
        self.move_information & 0x400000 != 0
    }

    pub fn castling(&self) -> bool {
        self.move_information & 0x800000 != 0
    }
}

pub fn generate_moves(attack_tables: &AttackTables, game: &Game) -> MoveList {
    let mut move_list = MoveList::new();

    let side = game.side_to_move();

    game.side_bitboards(side)
        .iter()
        .for_each(|(mut bitboard, piece)| {
            while let Some(source_square_index) = bitboard.get_lsb_index() {
                let source_square = Square::new_from_index(source_square_index);

                let attacks = generate_attacks(attack_tables, game, piece, &source_square);

                let mut generated_moves = match piece {
                    Piece::Pawn => {
                        let attack_table = attack_tables.attack_table(
                            &game.board(None),
                            piece,
                            side,
                            &source_square,
                        );

                        generate_pawn_moves(attack_table, attacks, game, source_square_index)
                    }
                    Piece::Knight | Piece::Bishop | Piece::Rook | Piece::Queen => {
                        generate_piece_moves(attacks, game, piece, side, &source_square)
                    }
                    Piece::King => {
                        let mut king_moves = MoveList::new();
                        king_moves.append_moves(&mut generate_piece_moves(
                            attacks,
                            game,
                            piece,
                            side,
                            &source_square,
                        ));
                        king_moves.append_moves(&mut generate_castling_moves(attack_tables, game));

                        king_moves
                    }
                };

                move_list.append_moves(&mut generated_moves);

                bitboard.pop_bit(&Square::new_from_index(source_square_index));
            }
        });

    move_list
}

fn generate_pawn_moves(
    attack_table: Bitboard,
    mut attacks: Bitboard,
    game: &Game,
    source_square_index: usize,
) -> MoveList {
    let promotion_pieces = [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight];

    let mut move_list = MoveList::new();

    let second_rank = Bitboard::new(0xFF_0000_0000_0000);
    let seventh_rank = Bitboard::new(0xFF00);

    let side = game.side_to_move();

    let source_square = Square::new_from_index(source_square_index);
    let target_square = match side {
        Side::White => Square::new_from_index(source_square_index - 8),
        Side::Black => Square::new_from_index(source_square_index + 8),
    };

    let single_piece = Bitboard::from_square(&source_square);

    let piece_on_second_rank = second_rank & single_piece != 0u64;
    let piece_on_seventh_rank = seventh_rank & single_piece != 0u64;

    if ((*side == Side::White && piece_on_seventh_rank)
        || (*side == Side::Black && piece_on_second_rank))
        && game.piece_at_square(&target_square).is_none()
    {
        promotion_pieces.iter().for_each(|promoted_piece| {
            move_list.add_move(
                &source_square,
                &target_square,
                &Piece::Pawn,
                side,
                Some(promoted_piece),
                MoveType::Quiet,
            );
        });
    } else if game.piece_at_square(&target_square).is_none() {
        move_list.add_move(
            &source_square,
            &target_square,
            &Piece::Pawn,
            side,
            None,
            MoveType::Quiet,
        );
    }

    let single_push_target_square = target_square;
    let double_push_target_square = if *side == Side::White && piece_on_second_rank {
        Some(Square::new_from_index(source_square_index - 16))
    } else if *side == Side::Black && piece_on_seventh_rank {
        Some(Square::new_from_index(source_square_index + 16))
    } else {
        None
    };

    if let Some(target_square) = double_push_target_square {
        if game.piece_at_square(&single_push_target_square).is_none() {
            let target_square_empty = game.piece_at_square(&target_square).is_none();

            if target_square_empty {
                move_list.add_move(
                    &source_square,
                    &target_square,
                    &Piece::Pawn,
                    side,
                    None,
                    MoveType::DoublePawnPush,
                );
            }
        }
    }

    while let Some(target_square_index) = attacks.get_lsb_index() {
        let target_square = Square::new_from_index(target_square_index);

        if (*side == Side::White && piece_on_seventh_rank)
            || (*side == Side::Black && piece_on_second_rank)
        {
            promotion_pieces.iter().for_each(|promoted_piece| {
                move_list.add_move(
                    &source_square,
                    &target_square,
                    &Piece::Pawn,
                    side,
                    Some(promoted_piece),
                    MoveType::Capture,
                );
            });
        } else {
            move_list.add_move(
                &source_square,
                &target_square,
                &Piece::Pawn,
                side,
                None,
                MoveType::Capture,
            );
        }

        attacks.pop_bit(&target_square);
    }

    if let Some(target_square) = game.en_passant_square() {
        let en_passant_square_attacked =
            attack_table & Bitboard::from_square(target_square) != 0u64;

        if en_passant_square_attacked {
            move_list.add_move(
                &source_square,
                target_square,
                &Piece::Pawn,
                side,
                None,
                MoveType::EnPassant,
            );
        }
    }

    move_list
}

fn generate_piece_moves(
    mut attacks: Bitboard,
    game: &Game,
    piece: &Piece,
    side: &Side,
    source_square: &Square,
) -> MoveList {
    let mut move_list = MoveList::new();

    while let Some(target_square_index) = attacks.get_lsb_index() {
        let target_square = Square::new_from_index(target_square_index);

        let move_type = match game.piece_at_square(&target_square) {
            Some(_) => MoveType::Capture,
            None => MoveType::Quiet,
        };

        move_list.add_move(source_square, &target_square, piece, side, None, move_type);

        attacks.pop_bit(&target_square);
    }

    move_list
}

fn generate_castling_moves(attack_tables: &AttackTables, game: &Game) -> MoveList {
    let mut move_list = MoveList::new();

    let side = game.side_to_move();

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

    let d_file_square_attacked =
        game.is_square_attacked(attack_tables, &side.opponent_side(), &d_file_square);
    let e_file_square_attacked =
        game.is_square_attacked(attack_tables, &side.opponent_side(), &e_file_square);
    let f_file_square_attacked =
        game.is_square_attacked(attack_tables, &side.opponent_side(), &f_file_square);

    if game.piece_at_square(&f_file_square).is_none()
        && game.piece_at_square(&g_file_square).is_none()
        && !e_file_square_attacked
        && !f_file_square_attacked
        && game.castling_type_allowed(&short_castle)
    {
        move_list.add_move(
            &e_file_square,
            &g_file_square,
            &Piece::King,
            side,
            None,
            MoveType::Castling,
        );
    }

    if game.piece_at_square(&b_file_square).is_none()
        && game.piece_at_square(&c_file_square).is_none()
        && game.piece_at_square(&d_file_square).is_none()
        && !d_file_square_attacked
        && !e_file_square_attacked
        && game.castling_type_allowed(&long_castle)
    {
        move_list.add_move(
            &e_file_square,
            &c_file_square,
            &Piece::King,
            side,
            None,
            MoveType::Castling,
        );
    }

    move_list
}

fn generate_attacks(
    attack_tables: &AttackTables,
    game: &Game,
    piece: &Piece,
    source_square: &Square,
) -> Bitboard {
    let side = game.side_to_move();

    match piece {
        Piece::Pawn => {
            let attack_table =
                attack_tables.attack_table(&game.board(None), piece, side, source_square);
            let opponent_board = game.board(Some(&side.opponent_side()));

            attack_table & opponent_board
        }
        _ => {
            attack_tables.attack_table(&game.board(None), piece, side, source_square)
                & !game.board(Some(side))
        }
    }
}

#[cfg(test)]
#[allow(clippy::unusual_byte_groupings)]
mod tests {
    use super::*;

    #[test]
    fn move_encoding() {
        let mv = Move::new(
            &Square::D4,
            &Square::D5,
            &Piece::Pawn,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_encoding = 0b0000_1111_0000_011011_100011;

        assert_eq!(mv.move_information, desired_encoding);

        let mv = Move::new(
            &Square::D5,
            &Square::D4,
            &Piece::Pawn,
            &Side::Black,
            None,
            MoveType::Quiet,
        );
        let desired_encoding = 0b0000_1111_0110_100011_011011;

        assert_eq!(mv.move_information, desired_encoding);

        let mv = Move::new(
            &Square::D2,
            &Square::D4,
            &Piece::Pawn,
            &Side::White,
            None,
            MoveType::DoublePawnPush,
        );
        let desired_encoding = 0b0010_1111_0000_100011_110011;

        assert_eq!(mv.move_information, desired_encoding);

        let mv = Move::new(
            &Square::D4,
            &Square::E5,
            &Piece::Pawn,
            &Side::White,
            None,
            MoveType::Capture,
        );
        let desired_encoding = 0b0001_1111_0000_011100_100011;

        assert_eq!(mv.move_information, desired_encoding);

        let mv = Move::new(
            &Square::D5,
            &Square::E6,
            &Piece::Pawn,
            &Side::White,
            None,
            MoveType::EnPassant,
        );
        let desired_encoding = 0b0101_1111_0000_010100_011011;

        assert_eq!(mv.move_information, desired_encoding);

        let mv = Move::new(
            &Square::D7,
            &Square::D8,
            &Piece::Pawn,
            &Side::White,
            Some(&Piece::Queen),
            MoveType::Quiet,
        );
        let desired_encoding = 0b0000_0100_0000_000011_001011;

        assert_eq!(mv.move_information, desired_encoding);

        let mv = Move::new(
            &Square::E1,
            &Square::G1,
            &Piece::King,
            &Side::White,
            None,
            MoveType::Castling,
        );
        let desired_encoding = 0b1000_1111_0101_111110_111100;

        assert_eq!(mv.move_information, desired_encoding);
    }

    #[test]
    fn move_decoding() {
        let mv = Move::new(
            &Square::D4,
            &Square::D5,
            &Piece::Pawn,
            &Side::White,
            None,
            MoveType::Quiet,
        );

        assert_eq!(mv.source_square(), Square::D4);
        assert_eq!(mv.target_square(), Square::D5);
        assert_eq!(mv.piece(), Piece::Pawn);

        let mv = Move::new(
            &Square::D2,
            &Square::D4,
            &Piece::Pawn,
            &Side::White,
            None,
            MoveType::DoublePawnPush,
        );

        assert!(mv.double_pawn_push());

        let mv = Move::new(
            &Square::D4,
            &Square::E5,
            &Piece::Pawn,
            &Side::White,
            None,
            MoveType::Capture,
        );

        assert!(mv.capture());

        let mv = Move::new(
            &Square::D5,
            &Square::E6,
            &Piece::Pawn,
            &Side::White,
            None,
            MoveType::EnPassant,
        );

        assert!(mv.en_passant());

        let mv = Move::new(
            &Square::D7,
            &Square::D8,
            &Piece::Pawn,
            &Side::White,
            Some(&Piece::Queen),
            MoveType::Quiet,
        );

        assert_eq!(mv.promoted_piece(), Some(Piece::Queen));

        let mv = Move::new(
            &Square::E1,
            &Square::G1,
            &Piece::King,
            &Side::White,
            None,
            MoveType::Castling,
        );

        assert!(mv.castling());
    }

    #[test]
    fn single_pawn_push() {
        let white_game = Game::initialise("8/8/8/8/8/3P4/8/8 w - - 0 1");
        let black_game = Game::initialise("8/8/3p4/8/8/8/8/8 b - - 0 1");

        let white_square = Square::D3;
        let black_square = Square::D6;

        let attack_tables = AttackTables::initialise();

        let white_attack_table = attack_tables.attack_table(
            &white_game.board(None),
            &Piece::Pawn,
            &Side::White,
            &white_square,
        );
        let black_attack_table = attack_tables.attack_table(
            &black_game.board(None),
            &Piece::Pawn,
            &Side::Black,
            &black_square,
        );

        let white_attacks =
            generate_attacks(&attack_tables, &white_game, &Piece::Pawn, &white_square);
        let black_attacks =
            generate_attacks(&attack_tables, &black_game, &Piece::Pawn, &black_square);

        let white_moves = generate_pawn_moves(
            white_attack_table,
            white_attacks,
            &white_game,
            white_square.as_usize(),
        );
        let black_moves = generate_pawn_moves(
            black_attack_table,
            black_attacks,
            &black_game,
            black_square.as_usize(),
        );

        let white_pawn_push = Move::new(
            &Square::D3,
            &Square::D4,
            &Piece::Pawn,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let black_pawn_push = Move::new(
            &Square::D6,
            &Square::D5,
            &Piece::Pawn,
            &Side::Black,
            None,
            MoveType::Quiet,
        );

        let white_moves_correct =
            white_moves.moves.contains(&white_pawn_push) && white_moves.moves.len() == 1;
        let black_moves_correct =
            black_moves.moves.contains(&black_pawn_push) && black_moves.moves.len() == 1;

        assert!(white_moves_correct);
        assert!(black_moves_correct);

        // blocked from pushing
        let white_game = Game::initialise("8/8/8/8/3p4/3P4/8/8 w - - 0 1");
        let black_game = Game::initialise("8/8/3p4/3P4/8/8/8/8 b - - 0 1");

        let white_square = Square::D3;
        let black_square = Square::D6;

        let attack_tables = AttackTables::initialise();

        let white_attack_table = attack_tables.attack_table(
            &white_game.board(None),
            &Piece::Pawn,
            &Side::White,
            &white_square,
        );
        let black_attack_table = attack_tables.attack_table(
            &black_game.board(None),
            &Piece::Pawn,
            &Side::Black,
            &black_square,
        );

        let white_attacks =
            generate_attacks(&attack_tables, &white_game, &Piece::Pawn, &white_square);
        let black_attacks =
            generate_attacks(&attack_tables, &black_game, &Piece::Pawn, &black_square);

        let white_moves = generate_pawn_moves(
            white_attack_table,
            white_attacks,
            &white_game,
            white_square.as_usize(),
        );
        let black_moves = generate_pawn_moves(
            black_attack_table,
            black_attacks,
            &black_game,
            black_square.as_usize(),
        );

        let white_moves_correct = white_moves.moves.is_empty();
        let black_moves_correct = black_moves.moves.is_empty();

        assert!(white_moves_correct);
        assert!(black_moves_correct);
    }

    #[test]
    fn double_pawn_push() {
        let white_game = Game::initialise("8/8/8/8/8/8/3P4/8 w - - 0 1");
        let black_game = Game::initialise("8/3p4/8/8/8/8/8/8 b - - 0 1");

        let white_square = Square::D2;
        let black_square = Square::D7;

        let attack_tables = AttackTables::initialise();

        let white_attack_table = attack_tables.attack_table(
            &white_game.board(None),
            &Piece::Pawn,
            &Side::White,
            &white_square,
        );
        let black_attack_table = attack_tables.attack_table(
            &black_game.board(None),
            &Piece::Pawn,
            &Side::Black,
            &black_square,
        );

        let white_attacks =
            generate_attacks(&attack_tables, &white_game, &Piece::Pawn, &white_square);
        let black_attacks =
            generate_attacks(&attack_tables, &black_game, &Piece::Pawn, &black_square);

        let white_moves = generate_pawn_moves(
            white_attack_table,
            white_attacks,
            &white_game,
            white_square.as_usize(),
        );
        let black_moves = generate_pawn_moves(
            black_attack_table,
            black_attacks,
            &black_game,
            black_square.as_usize(),
        );

        let white_single_pawn_push = Move::new(
            &Square::D2,
            &Square::D3,
            &Piece::Pawn,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let white_double_pawn_push = Move::new(
            &Square::D2,
            &Square::D4,
            &Piece::Pawn,
            &Side::White,
            None,
            MoveType::DoublePawnPush,
        );

        let black_single_pawn_push = Move::new(
            &Square::D7,
            &Square::D6,
            &Piece::Pawn,
            &Side::Black,
            None,
            MoveType::Quiet,
        );
        let black_double_pawn_push = Move::new(
            &Square::D7,
            &Square::D5,
            &Piece::Pawn,
            &Side::Black,
            None,
            MoveType::DoublePawnPush,
        );

        let white_moves_correct = white_moves.moves.contains(&white_single_pawn_push)
            && white_moves.moves.contains(&white_double_pawn_push)
            && white_moves.moves.len() == 2;
        let black_moves_correct = black_moves.moves.contains(&black_single_pawn_push)
            && black_moves.moves.contains(&black_double_pawn_push)
            && black_moves.moves.len() == 2;

        assert!(white_moves_correct);
        assert!(black_moves_correct);

        // blocked from double push
        let white_game = Game::initialise("8/8/8/8/3p4/8/3P4/8 w - - 0 1");
        let black_game = Game::initialise("8/3p4/8/3P4/8/8/8/8 b - - 0 1");

        let white_moves = generate_pawn_moves(
            white_attack_table,
            white_attacks,
            &white_game,
            white_square.as_usize(),
        );
        let black_moves = generate_pawn_moves(
            black_attack_table,
            black_attacks,
            &black_game,
            black_square.as_usize(),
        );

        let white_moves_correct =
            white_moves.moves.contains(&white_single_pawn_push) && white_moves.moves.len() == 1;
        let black_moves_correct =
            black_moves.moves.contains(&black_single_pawn_push) && black_moves.moves.len() == 1;

        assert!(white_moves_correct);
        assert!(black_moves_correct);

        // blocked from single push
        let white_game = Game::initialise("8/8/8/8/8/3p4/3P4/8 w - - 0 1");
        let black_game = Game::initialise("8/3p4/3P4/8/8/8/8/8 b - - 0 1");

        let white_moves = generate_pawn_moves(
            white_attack_table,
            white_attacks,
            &white_game,
            white_square.as_usize(),
        );
        let black_moves = generate_pawn_moves(
            black_attack_table,
            black_attacks,
            &black_game,
            black_square.as_usize(),
        );

        let white_moves_correct = white_moves.moves.is_empty();
        let black_moves_correct = black_moves.moves.is_empty();

        assert!(white_moves_correct);
        assert!(black_moves_correct);
    }

    #[test]
    fn pawn_capture() {
        let white_game = Game::initialise("8/8/8/2P1p3/3P4/8/8/8 w - - 0 1");
        let black_game = Game::initialise("8/8/8/3p4/2p1P3/8/8/8 b - - 0 1");

        let white_square = Square::D4;
        let black_square = Square::D5;

        let attack_tables = AttackTables::initialise();

        let white_attack_table = attack_tables.attack_table(
            &white_game.board(None),
            &Piece::Pawn,
            &Side::White,
            &white_square,
        );
        let black_attack_table = attack_tables.attack_table(
            &black_game.board(None),
            &Piece::Pawn,
            &Side::Black,
            &black_square,
        );

        let white_attacks =
            generate_attacks(&attack_tables, &white_game, &Piece::Pawn, &white_square);
        let black_attacks =
            generate_attacks(&attack_tables, &black_game, &Piece::Pawn, &black_square);

        let white_moves = generate_pawn_moves(
            white_attack_table,
            white_attacks,
            &white_game,
            white_square.as_usize(),
        );
        let black_moves = generate_pawn_moves(
            black_attack_table,
            black_attacks,
            &black_game,
            black_square.as_usize(),
        );

        let white_capture = Move::new(
            &Square::D4,
            &Square::E5,
            &Piece::Pawn,
            &Side::White,
            None,
            MoveType::Capture,
        );
        let black_capture = Move::new(
            &Square::D5,
            &Square::E4,
            &Piece::Pawn,
            &Side::Black,
            None,
            MoveType::Capture,
        );

        assert!(white_moves.moves.contains(&white_capture));
        assert!(black_moves.moves.contains(&black_capture));
    }

    #[test]
    fn pawn_promotion() {
        let white_game = Game::initialise("8/3P4/8/8/8/8/8/8 w - - 0 1");
        let black_game = Game::initialise("8/8/8/8/8/8/3p4/8 b - - 0 1");

        let white_square = Square::D7;
        let black_square = Square::D2;

        let attack_tables = AttackTables::initialise();

        let white_attack_table = attack_tables.attack_table(
            &white_game.board(None),
            &Piece::Pawn,
            &Side::White,
            &white_square,
        );
        let black_attack_table = attack_tables.attack_table(
            &black_game.board(None),
            &Piece::Pawn,
            &Side::Black,
            &black_square,
        );

        let white_attacks =
            generate_attacks(&attack_tables, &white_game, &Piece::Pawn, &white_square);
        let black_attacks =
            generate_attacks(&attack_tables, &black_game, &Piece::Pawn, &black_square);

        let white_moves = generate_pawn_moves(
            white_attack_table,
            white_attacks,
            &white_game,
            white_square.as_usize(),
        );
        let black_moves = generate_pawn_moves(
            black_attack_table,
            black_attacks,
            &black_game,
            black_square.as_usize(),
        );

        let white_promotion_queen = Move::new(
            &Square::D7,
            &Square::D8,
            &Piece::Pawn,
            &Side::White,
            Some(&Piece::Queen),
            MoveType::Quiet,
        );
        let white_promotion_rook = Move::new(
            &Square::D7,
            &Square::D8,
            &Piece::Pawn,
            &Side::White,
            Some(&Piece::Rook),
            MoveType::Quiet,
        );
        let white_promotion_bishop = Move::new(
            &Square::D7,
            &Square::D8,
            &Piece::Pawn,
            &Side::White,
            Some(&Piece::Bishop),
            MoveType::Quiet,
        );
        let white_promotion_knight = Move::new(
            &Square::D7,
            &Square::D8,
            &Piece::Pawn,
            &Side::White,
            Some(&Piece::Knight),
            MoveType::Quiet,
        );

        let black_promotion_queen = Move::new(
            &Square::D2,
            &Square::D1,
            &Piece::Pawn,
            &Side::Black,
            Some(&Piece::Queen),
            MoveType::Quiet,
        );
        let black_promotion_rook = Move::new(
            &Square::D2,
            &Square::D1,
            &Piece::Pawn,
            &Side::Black,
            Some(&Piece::Rook),
            MoveType::Quiet,
        );
        let black_promotion_bishop = Move::new(
            &Square::D2,
            &Square::D1,
            &Piece::Pawn,
            &Side::Black,
            Some(&Piece::Bishop),
            MoveType::Quiet,
        );
        let black_promotion_knight = Move::new(
            &Square::D2,
            &Square::D1,
            &Piece::Pawn,
            &Side::Black,
            Some(&Piece::Knight),
            MoveType::Quiet,
        );

        let white_moves_correct = white_moves.moves.contains(&white_promotion_queen)
            && white_moves.moves.contains(&white_promotion_rook)
            && white_moves.moves.contains(&white_promotion_bishop)
            && white_moves.moves.contains(&white_promotion_knight)
            && white_moves.moves.len() == 4;
        let black_moves_correct = black_moves.moves.contains(&black_promotion_queen)
            && black_moves.moves.contains(&black_promotion_rook)
            && black_moves.moves.contains(&black_promotion_bishop)
            && black_moves.moves.contains(&black_promotion_knight)
            && black_moves.moves.len() == 4;

        assert!(white_moves_correct);
        assert!(black_moves_correct);
    }

    #[test]
    fn en_passant() {
        let white_game = Game::initialise("8/8/8/3Pp3/8/8/8/8 w - e6 0 1");
        let black_game = Game::initialise("8/8/8/8/3pP3/8/8/8 b - e3 0 1");

        let white_square = Square::D5;
        let black_square = Square::D4;

        let attack_tables = AttackTables::initialise();

        let white_attack_table = attack_tables.attack_table(
            &white_game.board(None),
            &Piece::Pawn,
            &Side::White,
            &white_square,
        );
        let black_attack_table = attack_tables.attack_table(
            &black_game.board(None),
            &Piece::Pawn,
            &Side::Black,
            &black_square,
        );

        let white_attacks =
            generate_attacks(&attack_tables, &white_game, &Piece::Pawn, &white_square);
        let black_attacks =
            generate_attacks(&attack_tables, &black_game, &Piece::Pawn, &black_square);

        let white_moves = generate_pawn_moves(
            white_attack_table,
            white_attacks,
            &white_game,
            white_square.as_usize(),
        );
        let black_moves = generate_pawn_moves(
            black_attack_table,
            black_attacks,
            &black_game,
            black_square.as_usize(),
        );

        let white_en_passant = Move::new(
            &Square::D5,
            &Square::E6,
            &Piece::Pawn,
            &Side::White,
            None,
            MoveType::EnPassant,
        );
        let black_en_passant = Move::new(
            &Square::D4,
            &Square::E3,
            &Piece::Pawn,
            &Side::Black,
            None,
            MoveType::EnPassant,
        );

        let white_moves_correct = white_moves.moves.contains(&white_en_passant);
        let black_moves_correct = black_moves.moves.contains(&black_en_passant);

        assert!(white_moves_correct);
        assert!(black_moves_correct);
    }

    #[test]
    fn knight_moves() {
        let game = Game::initialise("8/8/2p5/5P2/3N4/1p6/2p1P3/8 w - - 0 1");

        let source_square = Square::D4;

        let attack_tables = AttackTables::initialise();

        let attacks = generate_attacks(&attack_tables, &game, &Piece::Knight, &source_square);

        let moves = generate_piece_moves(attacks, &game, &Piece::Knight, &Side::White, &Square::D4);

        let desired_c6_move = Move::new(
            &Square::D4,
            &Square::C6,
            &Piece::Knight,
            &Side::White,
            None,
            MoveType::Capture,
        );
        let desired_e6_move = Move::new(
            &Square::D4,
            &Square::E6,
            &Piece::Knight,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_b5_move = Move::new(
            &Square::D4,
            &Square::B5,
            &Piece::Knight,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_b3_move = Move::new(
            &Square::D4,
            &Square::B3,
            &Piece::Knight,
            &Side::White,
            None,
            MoveType::Capture,
        );
        let desired_f3_move = Move::new(
            &Square::D4,
            &Square::F3,
            &Piece::Knight,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_c2_move = Move::new(
            &Square::D4,
            &Square::C2,
            &Piece::Knight,
            &Side::White,
            None,
            MoveType::Capture,
        );

        let knight_moves_correct = moves.moves.contains(&desired_c6_move)
            && moves.moves.contains(&desired_e6_move)
            && moves.moves.contains(&desired_b5_move)
            && moves.moves.contains(&desired_b3_move)
            && moves.moves.contains(&desired_f3_move)
            && moves.moves.contains(&desired_c2_move)
            && moves.moves.len() == 6;

        assert!(knight_moves_correct);
    }

    #[test]
    fn bishop_moves() {
        let game = Game::initialise("8/6p1/8/8/3B4/8/5P2/8 w - - 0 1");

        let source_square = Square::D4;

        let attack_tables = AttackTables::initialise();

        let attacks = generate_attacks(&attack_tables, &game, &Piece::Bishop, &source_square);

        let moves = generate_piece_moves(attacks, &game, &Piece::Bishop, &Side::White, &Square::D4);

        let desired_a7_move = Move::new(
            &Square::D4,
            &Square::A7,
            &Piece::Bishop,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_b6_move = Move::new(
            &Square::D4,
            &Square::B6,
            &Piece::Bishop,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_c5_move = Move::new(
            &Square::D4,
            &Square::C5,
            &Piece::Bishop,
            &Side::White,
            None,
            MoveType::Quiet,
        );

        let desired_g7_move = Move::new(
            &Square::D4,
            &Square::G7,
            &Piece::Bishop,
            &Side::White,
            None,
            MoveType::Capture,
        );
        let desired_f6_move = Move::new(
            &Square::D4,
            &Square::F6,
            &Piece::Bishop,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_e5_move = Move::new(
            &Square::D4,
            &Square::E5,
            &Piece::Bishop,
            &Side::White,
            None,
            MoveType::Quiet,
        );

        let desired_c3_move = Move::new(
            &Square::D4,
            &Square::C3,
            &Piece::Bishop,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_b2_move = Move::new(
            &Square::D4,
            &Square::B2,
            &Piece::Bishop,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_a1_move = Move::new(
            &Square::D4,
            &Square::A1,
            &Piece::Bishop,
            &Side::White,
            None,
            MoveType::Quiet,
        );

        let desired_e3_move = Move::new(
            &Square::D4,
            &Square::E3,
            &Piece::Bishop,
            &Side::White,
            None,
            MoveType::Quiet,
        );

        let bishop_moves_correct = moves.moves.contains(&desired_a7_move)
            && moves.moves.contains(&desired_b6_move)
            && moves.moves.contains(&desired_c5_move)
            && moves.moves.contains(&desired_g7_move)
            && moves.moves.contains(&desired_f6_move)
            && moves.moves.contains(&desired_e5_move)
            && moves.moves.contains(&desired_c3_move)
            && moves.moves.contains(&desired_b2_move)
            && moves.moves.contains(&desired_a1_move)
            && moves.moves.contains(&desired_e3_move)
            && moves.moves.len() == 10;

        assert!(bishop_moves_correct);
    }

    #[test]
    fn rook_moves() {
        let game = Game::initialise("3p4/8/8/8/3R1P2/8/8/8 w - - 0 1");

        let source_square = Square::D4;

        let attack_tables = AttackTables::initialise();

        let attacks = generate_attacks(&attack_tables, &game, &Piece::Rook, &source_square);

        let moves = generate_piece_moves(attacks, &game, &Piece::Rook, &Side::White, &Square::D4);

        let desired_d8_move = Move::new(
            &Square::D4,
            &Square::D8,
            &Piece::Rook,
            &Side::White,
            None,
            MoveType::Capture,
        );
        let desired_d7_move = Move::new(
            &Square::D4,
            &Square::D7,
            &Piece::Rook,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_d6_move = Move::new(
            &Square::D4,
            &Square::D6,
            &Piece::Rook,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_d5_move = Move::new(
            &Square::D4,
            &Square::D5,
            &Piece::Rook,
            &Side::White,
            None,
            MoveType::Quiet,
        );

        let desired_a4_move = Move::new(
            &Square::D4,
            &Square::A4,
            &Piece::Rook,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_b4_move = Move::new(
            &Square::D4,
            &Square::B4,
            &Piece::Rook,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_c4_move = Move::new(
            &Square::D4,
            &Square::C4,
            &Piece::Rook,
            &Side::White,
            None,
            MoveType::Quiet,
        );

        let desired_e4_move = Move::new(
            &Square::D4,
            &Square::E4,
            &Piece::Rook,
            &Side::White,
            None,
            MoveType::Quiet,
        );

        let desired_d3_move = Move::new(
            &Square::D4,
            &Square::D3,
            &Piece::Rook,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_d2_move = Move::new(
            &Square::D4,
            &Square::D2,
            &Piece::Rook,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_d1_move = Move::new(
            &Square::D4,
            &Square::D1,
            &Piece::Rook,
            &Side::White,
            None,
            MoveType::Quiet,
        );

        let rook_moves_correct = moves.moves.contains(&desired_d8_move)
            && moves.moves.contains(&desired_d7_move)
            && moves.moves.contains(&desired_d6_move)
            && moves.moves.contains(&desired_d5_move)
            && moves.moves.contains(&desired_a4_move)
            && moves.moves.contains(&desired_b4_move)
            && moves.moves.contains(&desired_c4_move)
            && moves.moves.contains(&desired_e4_move)
            && moves.moves.contains(&desired_d3_move)
            && moves.moves.contains(&desired_d2_move)
            && moves.moves.contains(&desired_d1_move)
            && moves.moves.len() == 11;

        assert!(rook_moves_correct)
    }

    #[test]
    fn queen_moves() {
        let game = Game::initialise("3p4/6p1/8/8/3Q1P2/8/5P2/8 w - - 0 1");

        let source_square = Square::D4;

        let attack_tables = AttackTables::initialise();

        let attacks = generate_attacks(&attack_tables, &game, &Piece::Queen, &source_square);

        let moves = generate_piece_moves(attacks, &game, &Piece::Queen, &Side::White, &Square::D4);

        let desired_a7_move = Move::new(
            &Square::D4,
            &Square::A7,
            &Piece::Queen,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_b6_move = Move::new(
            &Square::D4,
            &Square::B6,
            &Piece::Queen,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_c5_move = Move::new(
            &Square::D4,
            &Square::C5,
            &Piece::Queen,
            &Side::White,
            None,
            MoveType::Quiet,
        );

        let desired_d8_move = Move::new(
            &Square::D4,
            &Square::D8,
            &Piece::Queen,
            &Side::White,
            None,
            MoveType::Capture,
        );
        let desired_d7_move = Move::new(
            &Square::D4,
            &Square::D7,
            &Piece::Queen,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_d6_move = Move::new(
            &Square::D4,
            &Square::D6,
            &Piece::Queen,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_d5_move = Move::new(
            &Square::D4,
            &Square::D5,
            &Piece::Queen,
            &Side::White,
            None,
            MoveType::Quiet,
        );

        let desired_g7_move = Move::new(
            &Square::D4,
            &Square::G7,
            &Piece::Queen,
            &Side::White,
            None,
            MoveType::Capture,
        );
        let desired_f6_move = Move::new(
            &Square::D4,
            &Square::F6,
            &Piece::Queen,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_e5_move = Move::new(
            &Square::D4,
            &Square::E5,
            &Piece::Queen,
            &Side::White,
            None,
            MoveType::Quiet,
        );

        let desired_a4_move = Move::new(
            &Square::D4,
            &Square::A4,
            &Piece::Queen,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_b4_move = Move::new(
            &Square::D4,
            &Square::B4,
            &Piece::Queen,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_c4_move = Move::new(
            &Square::D4,
            &Square::C4,
            &Piece::Queen,
            &Side::White,
            None,
            MoveType::Quiet,
        );

        let desired_e4_move = Move::new(
            &Square::D4,
            &Square::E4,
            &Piece::Queen,
            &Side::White,
            None,
            MoveType::Quiet,
        );

        let desired_c3_move = Move::new(
            &Square::D4,
            &Square::C3,
            &Piece::Queen,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_b2_move = Move::new(
            &Square::D4,
            &Square::B2,
            &Piece::Queen,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_a1_move = Move::new(
            &Square::D4,
            &Square::A1,
            &Piece::Queen,
            &Side::White,
            None,
            MoveType::Quiet,
        );

        let desired_d3_move = Move::new(
            &Square::D4,
            &Square::D3,
            &Piece::Queen,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_d2_move = Move::new(
            &Square::D4,
            &Square::D2,
            &Piece::Queen,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_d1_move = Move::new(
            &Square::D4,
            &Square::D1,
            &Piece::Queen,
            &Side::White,
            None,
            MoveType::Quiet,
        );

        let desired_e3_move = Move::new(
            &Square::D4,
            &Square::E3,
            &Piece::Queen,
            &Side::White,
            None,
            MoveType::Quiet,
        );

        let queen_moves_correct = moves.moves.contains(&desired_a7_move)
            && moves.moves.contains(&desired_b6_move)
            && moves.moves.contains(&desired_c5_move)
            && moves.moves.contains(&desired_d8_move)
            && moves.moves.contains(&desired_d7_move)
            && moves.moves.contains(&desired_d6_move)
            && moves.moves.contains(&desired_d5_move)
            && moves.moves.contains(&desired_g7_move)
            && moves.moves.contains(&desired_f6_move)
            && moves.moves.contains(&desired_e5_move)
            && moves.moves.contains(&desired_a4_move)
            && moves.moves.contains(&desired_b4_move)
            && moves.moves.contains(&desired_c4_move)
            && moves.moves.contains(&desired_e4_move)
            && moves.moves.contains(&desired_c3_move)
            && moves.moves.contains(&desired_b2_move)
            && moves.moves.contains(&desired_a1_move)
            && moves.moves.contains(&desired_d3_move)
            && moves.moves.contains(&desired_d2_move)
            && moves.moves.contains(&desired_d1_move)
            && moves.moves.contains(&desired_e3_move)
            && moves.moves.len() == 21;

        assert!(queen_moves_correct);
    }

    #[test]
    fn king_moves() {
        let game = Game::initialise("8/8/8/2pP4/2PK4/2p5/8/8 w - - 0 1");

        let source_square = Square::D4;

        let attack_tables = AttackTables::initialise();

        let attacks = generate_attacks(&attack_tables, &game, &Piece::King, &source_square);

        let moves = generate_piece_moves(attacks, &game, &Piece::King, &Side::White, &Square::D4);

        let desired_c5_move = Move::new(
            &Square::D4,
            &Square::C5,
            &Piece::King,
            &Side::White,
            None,
            MoveType::Capture,
        );
        let desired_e5_move = Move::new(
            &Square::D4,
            &Square::E5,
            &Piece::King,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_e4_move = Move::new(
            &Square::D4,
            &Square::E4,
            &Piece::King,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_c3_move = Move::new(
            &Square::D4,
            &Square::C3,
            &Piece::King,
            &Side::White,
            None,
            MoveType::Capture,
        );
        let desired_d3_move = Move::new(
            &Square::D4,
            &Square::D3,
            &Piece::King,
            &Side::White,
            None,
            MoveType::Quiet,
        );
        let desired_e3_move = Move::new(
            &Square::D4,
            &Square::E3,
            &Piece::King,
            &Side::White,
            None,
            MoveType::Quiet,
        );

        let king_moves_correct = moves.moves.contains(&desired_c5_move)
            && moves.moves.contains(&desired_e5_move)
            && moves.moves.contains(&desired_e4_move)
            && moves.moves.contains(&desired_c3_move)
            && moves.moves.contains(&desired_d3_move)
            && moves.moves.contains(&desired_e3_move)
            && moves.moves.len() == 6;

        assert!(king_moves_correct);
    }

    #[test]
    fn castling() {
        let game = Game::initialise("8/8/8/8/8/8/8/R3K2R w KQ - 0 1");

        let attack_tables = AttackTables::initialise();

        let moves = generate_castling_moves(&attack_tables, &game);

        let desired_short_castle = Move::new(
            &Square::E1,
            &Square::G1,
            &Piece::King,
            &Side::White,
            None,
            MoveType::Castling,
        );
        let desired_long_castle = Move::new(
            &Square::E1,
            &Square::C1,
            &Piece::King,
            &Side::White,
            None,
            MoveType::Castling,
        );

        let castling_moves_correct = moves.moves.contains(&desired_short_castle)
            && moves.moves.contains(&desired_long_castle)
            && moves.moves.len() == 2;

        assert!(castling_moves_correct);

        let game = Game::initialise("8/8/8/8/8/5r2/8/R3K2R w KQ - 0 1");

        let moves = generate_castling_moves(&attack_tables, &game);

        let castling_moves_correct =
            moves.moves.contains(&desired_long_castle) && moves.moves.len() == 1;

        assert!(castling_moves_correct);

        let game = Game::initialise("8/8/8/8/8/5q2/8/R3K2R w KQ - 0 1");

        let moves = generate_castling_moves(&attack_tables, &game);

        assert!(moves.moves.is_empty());

        let game = Game::initialise("8/8/8/8/8/8/8/R3K2R w - - 0 1");

        let moves = generate_castling_moves(&attack_tables, &game);

        assert!(moves.moves.is_empty());
    }
}
