use super::{
    attack_tables::AttackTables,
    game::{CastlingType, Game},
    Bitboard, BoardSquare, EnumToInt, Piece, Side,
};
use core::panic;

static BLACK_PIECE_OFFSET: u32 = 6;
static NO_PIECE_VALUE: u32 = 0b1111;
static PROMOTION_PIECES: [Piece; 4] = [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight];

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

    fn add_move(&mut self, mv: Move) {
        self.moves.push(mv);
    }

    fn append_move_list(&mut self, move_list: &mut MoveList) {
        self.moves.append(&mut move_list.moves);
    }
}

pub struct Move {
    move_information: u32,
}

impl Move {
    fn new(
        (source_square, target_square): (&BoardSquare, &BoardSquare),
        (piece, side): (&Piece, &Side),
        promoted_piece: Option<&Piece>,
        capture: bool,
        double_pawn_push: bool,
        en_passant: bool,
        castling: bool,
    ) -> Self {
        let side_value_offset = if matches!(side, Side::Black) {
            BLACK_PIECE_OFFSET
        } else {
            0
        };
        let piece_value = piece.as_u32() + side_value_offset;
        let promoted_piece_value = if let Some(promoted_piece) = promoted_piece {
            match promoted_piece {
                Piece::Pawn | Piece::King => panic!("Attempted to promote pawn to pawn or king"),
                _ => promoted_piece.as_u32() + side_value_offset,
            }
        } else {
            NO_PIECE_VALUE
        };

        let mut move_information = source_square.as_u32();
        move_information |= target_square.as_u32() << 6;
        move_information |= piece_value << 12;
        move_information |= promoted_piece_value << 16;
        move_information |= (capture as u32) << 20;
        move_information |= (double_pawn_push as u32) << 21;
        move_information |= (en_passant as u32) << 22;
        move_information |= (castling as u32) << 23;

        Move { move_information }
    }

    pub fn source_square(&self) -> BoardSquare {
        let source_square_index = self.move_information & 0x3F;

        BoardSquare::new_from_index(source_square_index as usize)
    }

    pub fn target_square(&self) -> BoardSquare {
        let target_square_index = (self.move_information & 0xFC0) >> 6;

        BoardSquare::new_from_index(target_square_index as usize)
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
            None
        } else {
            Some(Piece::new_from_u32(promoted_piece_value))
        }
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

pub fn generate_moves(game: &Game) -> MoveList {
    let mut move_list = MoveList::new();

    let attack_tables = AttackTables::initialise();

    let side = game.side_to_move();

    game.side_to_move_bitboards()
        .iter()
        .for_each(|bitboard_info| {
            let (mut bitboard, piece) = bitboard_info;

            while let Some(source_square_index) = bitboard.get_ls1b_index() {
                let source_square = BoardSquare::new_from_index(source_square_index);

                let attacks = match piece {
                    Piece::Pawn => {
                        let attack_table = attack_tables.attack_table(
                            &game.board(None),
                            piece,
                            side,
                            &source_square,
                        );
                        let opponent_board = game.board(Some(&side.opponent_side()));

                        Bitboard::new(attack_table.bitboard & opponent_board.bitboard)
                    }
                    _ => Bitboard::new(
                        attack_tables
                            .attack_table(&game.board(None), piece, side, &source_square)
                            .bitboard
                            & !game.board(Some(side)).bitboard,
                    ),
                };

                match piece {
                    Piece::Pawn => {
                        let attack_table = attack_tables.attack_table(
                            &game.board(None),
                            piece,
                            side,
                            &source_square,
                        );
                        move_list.append_move_list(&mut generate_pawn_moves(
                            attack_table,
                            attacks,
                            game,
                            source_square_index,
                        ));
                    }
                    Piece::Knight | Piece::Bishop | Piece::Rook | Piece::Queen => {
                        move_list.append_move_list(&mut generate_piece_moves(
                            attacks,
                            game,
                            &source_square,
                        ));
                    }
                    Piece::King => {
                        move_list.append_move_list(&mut generate_piece_moves(
                            attacks,
                            game,
                            &source_square,
                        ));
                        move_list
                            .append_move_list(&mut generate_castling_moves(&attack_tables, game));
                    }
                }

                bitboard.pop_bit(&BoardSquare::new_from_index(source_square_index));
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
    let mut move_list = MoveList::new();

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

    if ((matches!(side, Side::White) && piece_on_seventh_rank)
        || (matches!(side, Side::Black) && piece_on_second_rank))
        && game.piece_at_square(&target_square).is_none()
    {
        PROMOTION_PIECES.iter().for_each(|promoted_piece| {
            let mv = Move::new(
                (&source_square, &target_square),
                (&Piece::Pawn, side),
                Some(promoted_piece),
                false,
                false,
                false,
                false,
            );
            move_list.add_move(mv);
        });
    } else if game.piece_at_square(&target_square).is_none() {
        let mv = Move::new(
            (&source_square, &target_square),
            (&Piece::Pawn, side),
            None,
            false,
            false,
            false,
            false,
        );
        move_list.add_move(mv);
    }

    let single_push_target_square = target_square;
    let double_push_target_square = if matches!(side, Side::White) && piece_on_second_rank {
        Some(BoardSquare::new_from_index(source_square_index - 16))
    } else if matches!(side, Side::Black) && piece_on_seventh_rank {
        Some(BoardSquare::new_from_index(source_square_index + 16))
    } else {
        None
    };

    if let Some(target_square) = double_push_target_square {
        if game.piece_at_square(&single_push_target_square).is_none() {
            let target_square_empty = game.piece_at_square(&target_square).is_none();

            if target_square_empty {
                let mv = Move::new(
                    (&source_square, &target_square),
                    (&Piece::Pawn, side),
                    None,
                    false,
                    true,
                    false,
                    false,
                );
                move_list.add_move(mv);
            }
        }
    }

    while let Some(target_square_index) = attacks.get_ls1b_index() {
        let target_square = BoardSquare::new_from_index(target_square_index);

        if (matches!(side, Side::White) && piece_on_seventh_rank)
            || (matches!(side, Side::Black) && piece_on_second_rank)
        {
            PROMOTION_PIECES.iter().for_each(|promoted_piece| {
                let mv = Move::new(
                    (&source_square, &target_square),
                    (&Piece::Pawn, side),
                    Some(promoted_piece),
                    true,
                    false,
                    false,
                    false,
                );
                move_list.add_move(mv);
            });
        } else {
            let mv = Move::new(
                (&source_square, &target_square),
                (&Piece::Pawn, side),
                None,
                true,
                false,
                false,
                false,
            );
            move_list.add_move(mv);
        }

        attacks.pop_bit(&target_square);
    }

    if let Some(target_square) = game.en_passant_square() {
        let en_passant_square_attacked =
            attack_table.bitboard & Bitboard::from_square(target_square).bitboard != 0;

        if en_passant_square_attacked {
            let mv = Move::new(
                (&source_square, target_square),
                (&Piece::Pawn, side),
                None,
                true,
                false,
                true,
                false,
            );
            move_list.add_move(mv);
        }
    }

    move_list
}

fn generate_piece_moves(
    mut attacks: Bitboard,
    game: &Game,
    source_square: &BoardSquare,
) -> MoveList {
    let mut move_list = MoveList::new();

    while let Some(target_square_index) = attacks.get_ls1b_index() {
        let target_square = BoardSquare::new_from_index(target_square_index);

        let (piece, side) = match game.piece_at_square(source_square) {
            Some((piece, side)) => (piece, side),
            None => panic!("Attempting to generate piece moves for empty source square"),
        };

        let capture_flag = matches!(game.piece_at_square(&target_square), Some(_));

        let mv = Move::new(
            (source_square, &target_square),
            (&piece, &side),
            None,
            capture_flag,
            false,
            false,
            false,
        );
        move_list.add_move(mv);
        attacks.pop_bit(&target_square);
    }

    move_list
}

fn generate_castling_moves(attack_tables: &AttackTables, game: &Game) -> MoveList {
    let mut move_list = MoveList::new();

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
        let mv = Move::new(
            (&e_file_square, &g_file_square),
            (&Piece::King, side),
            None,
            false,
            false,
            false,
            true,
        );
        move_list.add_move(mv);
    }

    if game.piece_at_square(&b_file_square).is_none()
        && game.piece_at_square(&c_file_square).is_none()
        && game.piece_at_square(&d_file_square).is_none()
        && !d_file_square_attacked
        && !e_file_square_attacked
        && game.castling_type_allowed(&long_castle)
    {
        let mv = Move::new(
            (&e_file_square, &c_file_square),
            (&Piece::King, side),
            None,
            false,
            false,
            false,
            true,
        );
        move_list.add_move(mv);
    }

    move_list
}
