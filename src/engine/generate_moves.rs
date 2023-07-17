use super::{
    attack_tables::AttackTables,
    game::{CastlingType, Game},
    Bitboard, BoardSquare, EnumToInt, Piece, Side,
};

static NONE_PIECE_VALUE: u32 = 0b1111;

struct Move {
    move_information: u32,
}

impl Move {
    pub fn new(
        source_square: &BoardSquare,
        target_square: &BoardSquare,
        (piece, side): (&Piece, &Side),
        promoted_piece: Option<&Piece>,
        capture: bool,
        double_push: bool,
        en_passant: bool,
        castling: bool,
    ) -> Self {
        let side_info = if matches!(side, Side::Black) { 6 } else { 0 };
        let piece_info = piece.as_u32() + side_info;
        let promoted_piece_info = if let Some(promoted_piece) = promoted_piece {
            promoted_piece.as_u32() + side_info
        } else {
            NONE_PIECE_VALUE
        };

        let mut move_information = source_square.as_u32();
        move_information |= target_square.as_u32() << 6;
        move_information |= piece_info << 12;
        move_information |= promoted_piece_info << 16;
        move_information |= (capture as u32) << 20;
        move_information |= (double_push as u32) << 21;
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
        let piece_index = (self.move_information & 0xF000) >> 12;

        Piece::new_from_u32(piece_index)
    }

    pub fn promoted_piece(&self) -> Option<Piece> {
        let promoted_piece_index = (self.move_information & 0xF0000) >> 16;

        if promoted_piece_index == NONE_PIECE_VALUE {
            None
        } else {
            Some(Piece::new_from_u32(promoted_piece_index))
        }
    }

    pub fn capture(&self) -> bool {
        self.move_information & 0x100000 != 0
    }

    pub fn double_push(&self) -> bool {
        self.move_information & 0x200000 != 0
    }

    pub fn en_passant(&self) -> bool {
        self.move_information & 0x400000 != 0
    }

    pub fn castling(&self) -> bool {
        self.move_information & 0x800000 != 0
    }
}

pub fn generate_moves(game: &Game) {
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

                        generate_pawn_attacks(
                            attack_table,
                            game.en_passant_square(),
                            game.board(Some(&side.opponent_side())),
                        )
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
                        generate_pawn_moves(attacks, game, source_square_index);
                    }
                    Piece::Knight | Piece::Bishop | Piece::Rook | Piece::Queen => {
                        generate_piece_moves(attacks, &source_square);
                    }
                    Piece::King => {
                        generate_piece_moves(attacks, &source_square);
                        generate_castling_moves(&attack_tables, game);
                    }
                }

                bitboard.pop_bit(&BoardSquare::new_from_index(source_square_index));
            }
        });
}

fn generate_pawn_moves(mut attacks: Bitboard, game: &Game, source_square_index: usize) {
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

            let target_square_string = target_square.to_lowercase_string();

            if target_square_empty {
                println!("{}{}", source_square_string, target_square_string);
            }
        }
    }

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
}

fn generate_piece_moves(mut attacks: Bitboard, source_square: &BoardSquare) {
    while let Some(target_square_index) = attacks.get_ls1b_index() {
        let target_square = BoardSquare::new_from_index(target_square_index);

        let source_square_string = source_square.to_lowercase_string();
        let target_square_string = target_square.to_lowercase_string();

        println!("{}{}", source_square_string, target_square_string);

        attacks.pop_bit(&target_square);
    }
}

fn generate_castling_moves(attack_tables: &AttackTables, game: &Game) {
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

fn generate_pawn_attacks(
    attack_table: Bitboard,
    en_passant_square: &Option<BoardSquare>,
    opponent_board: Bitboard,
) -> Bitboard {
    let en_passant = if let Some(en_passant_square) = en_passant_square {
        Bitboard::from_square(en_passant_square)
    } else {
        Bitboard::new(0)
    };

    Bitboard::new(attack_table.bitboard & (opponent_board.bitboard | en_passant.bitboard))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_encoding() {
        // let move = Move::new(BoardSquare::E2, BoardSquare::E4,Piece::Pawn,)
    }
}
