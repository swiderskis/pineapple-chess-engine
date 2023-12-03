use super::{game::Game, moves::MoveFlag, Engine};
use crate::uci::InputError;

// piece value obtained by indexing into array using Piece enum
const PIECE_VALUE: [Evaluation; 6] = [100, 300, 350, 500, 900, 10_000];

type Evaluation = i32;

impl Engine {
    pub fn find_best_move(&self) -> Result<String, InputError> {
        let side = self.game.side_to_move();

        let mut best_evaluation = -i32::MAX * side as i32;
        let mut best_move = None;

        for mv in self.move_list.move_list().iter().flatten() {
            let mut game_clone = self.game.clone();

            if game_clone
                .make_move(&self.attack_tables, mv, MoveFlag::All)
                .is_ok()
            {
                let current_evalution = Self::evaluate(&game_clone);

                if current_evalution * side as i32 > best_evaluation * side as i32 {
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
        let mut evaluation = 0;

        for (bitboard, piece, side) in game.piece_bitboards() {
            let piece_count = bitboard.count_bits() as i32;
            let piece_count_evalutation = piece_count * PIECE_VALUE[piece as usize] * side as i32;

            evaluation += piece_count_evalutation;
        }

        evaluation
    }
}
