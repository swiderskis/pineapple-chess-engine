use super::{
    game::{Game, Piece, Side},
    moves::MoveFlag,
    Engine,
};
use crate::uci::InputError;

// piece value obtained by indexing into array using Piece enum
const PIECE_VALUE: [i32; 6] = [100, 300, 350, 500, 900, 10_000];

// position values obtained by indexing in using Square enum
// indexing must be done with side taken into consideration
// if side is black, index (63 - square) should be used
const PAWN_POSITION_VALUE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 30, 30, 30, 40, 40, 30, 30, 30, 20, 20, 20, 30, 30, 30, 20, 20, 10, 10,
    10, 20, 20, 10, 10, 10, 5, 5, 10, 20, 20, 5, 5, 5, 0, 0, 0, 5, 5, 0, 0, 0, 0, 0, 0, -10, -10,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const KNIGHT_POSITION_VALUE: [i32; 64] = [
    -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 10, 10, 0, 0, -5, -5, 5, 20, 20, 20, 20, 5, -5, -5, 10, 20,
    30, 30, 20, 10, -5, -5, 10, 20, 30, 30, 20, 10, -5, -5, 5, 20, 10, 10, 20, 5, -5, -5, 0, 0, 0,
    0, 0, 0, -5, -5, -10, 0, 0, 0, 0, -10, -5,
];
const BISHOP_POSITION_VALUE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 10, 0, 0, 0, 0, 0, 10, 20, 20, 10,
    0, 0, 0, 0, 10, 20, 20, 10, 0, 0, 0, 10, 0, 0, 0, 0, 10, 0, 0, 30, 0, 0, 0, 0, 30, 0, 0, 0,
    -10, 0, 0, -10, 0, 0,
];
const ROOK_POSITION_VALUE: [i32; 64] = [
    50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 0, 0, 10, 20, 20, 10, 0, 0, 0,
    0, 10, 20, 20, 10, 0, 0, 0, 0, 10, 20, 20, 10, 0, 0, 0, 0, 10, 20, 20, 10, 0, 0, 0, 0, 10, 20,
    20, 10, 0, 0, 0, 0, 0, 20, 20, 0, 0, 0,
];
const KING_POSITION_VALUE: [i32; 64] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 5, 5, 0, 0, 0, 5, 5, 10, 10, 5, 5, 0, 0, 5, 10, 20, 20, 10,
    5, 0, 0, 5, 10, 20, 20, 10, 5, 0, 0, 0, 5, 10, 10, 5, 0, 0, 0, 5, 5, -5, -5, 0, 5, 0, 0, 0, 5,
    0, -15, 0, 10, 0,
];

struct Evaluation(i32);

impl Evaluation {
    fn add(&mut self, value: i32, side: Side) {
        self.0 += value * side as i32;
    }

    fn sided_value(&self, side: Side) -> i32 {
        self.0 * side as i32
    }
}

impl Engine {
    pub fn find_best_move(&self) -> Result<String, InputError> {
        let side = self.game.side_to_move();

        let mut best_evaluation = Evaluation(-i32::MAX * side as i32);
        let mut best_move = None;

        for mv in self.move_list.move_list().iter().flatten() {
            let mut game_clone = self.game.clone();

            if game_clone
                .make_move(&self.attack_tables, mv, MoveFlag::All)
                .is_ok()
            {
                let current_evalution = Self::evaluate(&game_clone);

                if current_evalution.sided_value(side) > best_evaluation.sided_value(side) {
                    best_evaluation = current_evalution;
                    best_move = Some(mv);
                }
            }
        }

        match best_move {
            Some(mv) => Ok(mv.as_string()),
            None => Err(InputError::UninitialisedPosition),
        }
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
                    Side::Black => 63 - square as usize,
                };

                evaluation.add(PIECE_VALUE[piece as usize], side);
                evaluation.add(position_value[sided_square_index], side);

                bitboard.pop_bit(square);
            }
        }

        evaluation
    }
}
