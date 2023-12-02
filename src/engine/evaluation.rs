use super::{
    game::{Game, Piece, Side},
    moves::MoveFlag,
    Engine,
};
use crate::uci::InputError;

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
            let piece_count_evalutation = piece_count * Self::piece_value(piece, side);

            evaluation += piece_count_evalutation;
        }

        evaluation
    }

    fn piece_value(piece: Piece, side: Side) -> Evaluation {
        let score = match piece {
            Piece::Pawn => 100,
            Piece::Knight => 300,
            Piece::Bishop => 350,
            Piece::Rook => 500,
            Piece::Queen => 900,
            Piece::King => 10_000,
        };

        score * side as Evaluation
    }
}
