use super::{
    attack_tables::{AttackTablesPub, LeaperAttackTables, SliderAttackTables},
    game::{CastlingType, Game},
    Bitboard, BoardSquare, Piece, Side,
};

pub fn generate_moves(game: &Game) {
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
                    Piece::Knight => generate_leaper_piece_moves(
                        source_square_index,
                        game,
                        &leaper_attack_tables,
                        &Piece::Knight,
                    ),
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

fn generate_leaper_piece_moves(
    source_square_index: usize,
    game: &Game,
    leaper_attack_tables: &LeaperAttackTables,
    piece: &Piece,
) {
    let side = game.side_to_move();

    let source_square = BoardSquare::new_from_index(source_square_index);

    let mut attacks = Bitboard::new(
        leaper_attack_tables
            .attack_table(&game.board(&Side::Either), piece, side, &source_square)
            .bitboard
            & !game.board(side).bitboard,
    );

    while let Some(target_square_index) = attacks.get_ls1b_index() {
        let target_square = BoardSquare::new_from_index(target_square_index);

        let source_square_string = source_square.to_lowercase_string();
        let target_square_string = target_square.to_lowercase_string();

        println!("{}{}", source_square_string, target_square_string);

        attacks.pop_bit(&target_square);
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
