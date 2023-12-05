use super::{
    game::{Game, Piece, Side},
    moves::{MoveFlag, MoveList},
    Engine,
};
use crate::uci::InputError;
use std::ops::Neg;

// piece value obtained by indexing into array using Piece enum
const PIECE_VALUE: [i32; 6] = [100, 300, 350, 500, 900, 10_000];

// position values obtained by indexing in using Square enum
// indexing must be done with side taken into consideration
// if side is black, index (63 - square) should be used
#[rustfmt::skip]
const PAWN_POSITION_VALUE: [i32; 64] = [
     0,  0,  0,   0,   0,  0,  0,  0,
    30, 30, 30,  40,  40, 30, 30, 30,
    20, 20, 20,  30,  30, 30, 20, 20,
    10, 10, 10,  20,  20, 10, 10, 10,
     5,  5, 10,  20,  20,  5,  5,  5,
     0,  0,  0,   5,   5,  0,  0,  0,
     0,  0,  0, -10, -10,  0,  0,  0,
     0,  0,  0,   0,   0,  0,  0,  0,
];
#[rustfmt::skip]
const KNIGHT_POSITION_VALUE: [i32; 64] = [
    -5,   0,  0,  0,  0,  0,   0, -5,
    -5,   0,  0, 10, 10,  0,   0, -5,
    -5,   5, 20, 20, 20, 20,   5, -5,
    -5,  10, 20, 30, 30, 20,  10, -5,
    -5,  10, 20, 30, 30, 20,  10, -5,
    -5,   5, 20, 10, 10, 20,   5, -5,
    -5,   0,  0,  0,  0,  0,   0, -5,
    -5, -10,  0,  0,  0,  0, -10, -5,
];
#[rustfmt::skip]
const BISHOP_POSITION_VALUE: [i32; 64] = [
    0,  0,   0,  0,  0,   0,  0, 0,
    0,  0,   0,  0,  0,   0,  0, 0,
    0,  0,   0, 10, 10,   0,  0, 0,
    0,  0,  10, 20, 20,  10,  0, 0,
    0,  0,  10, 20, 20,  10,  0, 0,
    0, 10,   0,  0,  0,   0, 10, 0,
    0, 30,   0,  0,  0,   0, 30, 0,
    0,  0, -10,  0,  0, -10,  0, 0,
];
#[rustfmt::skip]
const ROOK_POSITION_VALUE: [i32; 64] = [
    50, 50, 50, 50, 50, 50, 50, 50,
    50, 50, 50, 50, 50, 50, 50, 50,
     0,  0, 10, 20, 20, 10,  0,  0,
     0,  0, 10, 20, 20, 10,  0,  0,
     0,  0, 10, 20, 20, 10,  0,  0,
     0,  0, 10, 20, 20, 10,  0,  0,
     0,  0, 10, 20, 20, 10,  0,  0,
     0,  0,  0, 20, 20,  0,  0,  0,
];
#[rustfmt::skip]
const KING_POSITION_VALUE: [i32; 64] = [
    0, 0,  0,  0,   0,  0,  0, 0,
    0, 0,  5,  5,   5,  5,  0, 0,
    0, 5,  5, 10,  10,  5,  5, 0,
    0, 5, 10, 20,  20, 10,  5, 0,
    0, 5, 10, 20,  20, 10,  5, 0,
    0, 0,  5, 10,  10,  5,  0, 0,
    0, 5,  5, -5,  -5,  0,  5, 0,
    0, 0,  5,  0, -15,  0, 10, 0,
];

#[derive(Clone, Copy, PartialEq, PartialOrd)]
struct Evaluation(i32);

impl Evaluation {
    fn add(&mut self, value: i32, side: Side) {
        self.0 += value * side as i32;
    }

    fn sided_value(&self, side: Side) -> Evaluation {
        Self(self.0 * side as i32)
    }
}

impl Neg for Evaluation {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl Engine {
    pub fn find_best_move(&self, depth: u8) -> Result<String, InputError> {
        let mut min_evaluation = Evaluation(-i32::MAX);
        let max_evaluation = Evaluation(i32::MAX);

        let mut best_move = None;

        for mv in self.move_list.move_list().iter().flatten() {
            let mut game_clone = self.game.clone();

            if game_clone
                .make_move(&self.attack_tables, mv, MoveFlag::All)
                .is_err()
            {
                continue;
            }

            let evaluation = -self.search(&game_clone, -max_evaluation, -min_evaluation, depth - 1);

            if evaluation > min_evaluation {
                min_evaluation = evaluation;
                best_move = Some(mv);
            }
        }

        match best_move {
            Some(mv) => Ok(mv.as_string()),
            None => Err(InputError::UninitialisedPosition),
        }
    }

    // negamax alpha beta search algorithm
    fn search(
        &self,
        game: &Game,
        mut min_evaluation: Evaluation,
        max_evaluation: Evaluation,
        depth: u8,
    ) -> Evaluation {
        if depth == 0 {
            return Self::evaluate(game);
        }

        let move_list = MoveList::generate_moves(&self.attack_tables, game);

        for mv in move_list.move_list().iter().flatten() {
            let mut game_clone = game.clone();

            if game_clone
                .make_move(&self.attack_tables, mv, MoveFlag::All)
                .is_err()
            {
                continue;
            }

            let evaluation = -self.search(&game_clone, -max_evaluation, -min_evaluation, depth - 1);

            if evaluation >= max_evaluation {
                return max_evaluation;
            }

            if evaluation > min_evaluation {
                min_evaluation = evaluation;
            }
        }

        min_evaluation
    }

    fn evaluate(game: &Game) -> Evaluation {
        let mut evaluation = Evaluation(0);

        for (mut bitboard, piece, side) in game.piece_bitboards() {
            let position_value = match piece {
                Piece::Pawn => PAWN_POSITION_VALUE,
                Piece::Knight => KNIGHT_POSITION_VALUE,
                Piece::Bishop => BISHOP_POSITION_VALUE,
                Piece::Rook => ROOK_POSITION_VALUE,
                Piece::Queen => [0; 64],
                Piece::King => KING_POSITION_VALUE,
            };

            while let Some(square) = bitboard.get_lsb_square() {
                let sided_square_index = match side {
                    Side::White => square as usize,
                    Side::Black => square.horizontal_mirror() as usize,
                };

                evaluation.add(PIECE_VALUE[piece as usize], side);
                evaluation.add(position_value[sided_square_index], side);

                bitboard.pop_bit(square);
            }
        }

        evaluation.sided_value(game.side_to_move())
    }
}
