use super::{
    game::{Game, Piece, Side, Square},
    moves::{Move, MoveList, MoveType},
    Engine,
};
use crate::uci::InputError;
use std::ops::Neg;

pub type Value = i16;

const MAX_EVALUATION_VALUE: Value = Value::MAX;
const CHECKMATE_VALUE: Value = Value::MAX - 1;
const STALEMATE_VALUE: Value = 0;

// Piece value obtained by indexing into array using Piece enum
const PIECE_VALUE: [Value; 6] = [100, 300, 350, 500, 900, 0];

#[rustfmt::skip]
const PAWN_POSITION_VALUE: PositionValue = PositionValue([
     0,  0,  0,   0,   0,  0,  0,  0,
    30, 30, 30,  40,  40, 30, 30, 30,
    20, 20, 20,  30,  30, 30, 20, 20,
    10, 10, 10,  20,  20, 10, 10, 10,
     5,  5, 10,  20,  20,  5,  5,  5,
     0,  0,  0,   5,   5,  0,  0,  0,
     0,  0,  0, -10, -10,  0,  0,  0,
     0,  0,  0,   0,   0,  0,  0,  0,
]);
#[rustfmt::skip]
const KNIGHT_POSITION_VALUE: PositionValue = PositionValue([
    -5,   0,  0,  0,  0,  0,   0, -5,
    -5,   0,  0, 10, 10,  0,   0, -5,
    -5,   5, 20, 20, 20, 20,   5, -5,
    -5,  10, 20, 30, 30, 20,  10, -5,
    -5,  10, 20, 30, 30, 20,  10, -5,
    -5,   5, 20, 10, 10, 20,   5, -5,
    -5,   0,  0,  0,  0,  0,   0, -5,
    -5, -10,  0,  0,  0,  0, -10, -5,
]);
#[rustfmt::skip]
const BISHOP_POSITION_VALUE: PositionValue = PositionValue([
    0,  0,   0,  0,  0,   0,  0, 0,
    0,  0,   0,  0,  0,   0,  0, 0,
    0,  0,   0, 10, 10,   0,  0, 0,
    0,  0,  10, 20, 20,  10,  0, 0,
    0,  0,  10, 20, 20,  10,  0, 0,
    0, 10,   0,  0,  0,   0, 10, 0,
    0, 30,   0,  0,  0,   0, 30, 0,
    0,  0, -10,  0,  0, -10,  0, 0,
]);
#[rustfmt::skip]
const ROOK_POSITION_VALUE: PositionValue = PositionValue([
    50, 50, 50, 50, 50, 50, 50, 50,
    50, 50, 50, 50, 50, 50, 50, 50,
     0,  0, 10, 20, 20, 10,  0,  0,
     0,  0, 10, 20, 20, 10,  0,  0,
     0,  0, 10, 20, 20, 10,  0,  0,
     0,  0, 10, 20, 20, 10,  0,  0,
     0,  0, 10, 20, 20, 10,  0,  0,
     0,  0,  0, 20, 20,  0,  0,  0,
]);
#[rustfmt::skip]
const KING_POSITION_VALUE: PositionValue = PositionValue([
    0, 0,  0,  0,   0,  0,  0, 0,
    0, 0,  5,  5,   5,  5,  0, 0,
    0, 5,  5, 10,  10,  5,  5, 0,
    0, 5, 10, 20,  20, 10,  5, 0,
    0, 5, 10, 20,  20, 10,  5, 0,
    0, 0,  5, 10,  10,  5,  0, 0,
    0, 5,  5, -5,  -5,  0,  5, 0,
    0, 0,  5,  0, -15,  0, 10, 0,
]);

impl Engine {
    pub fn find_best_move(&mut self, mut depth: u8) -> Result<Move, InputError> {
        let mut min_evaluation = -Evaluation(MAX_EVALUATION_VALUE);
        let max_evaluation = Evaluation(MAX_EVALUATION_VALUE);

        let mut total_nodes = 0;

        let king_square = self
            .game
            .piece_bitboard(Piece::King, self.game.side_to_move())
            .get_lsb_square();
        let king_in_check = match king_square {
            Some(king_square) => {
                let attacking_side = self.game.side_to_move().opponent_side();

                self.game
                    .is_square_attacked(&self.attack_tables, attacking_side, king_square)
            }
            None => false,
        };

        if king_in_check {
            depth += 1;
        }

        let mut best_move = None;

        let move_list = MoveList::generate_sorted_moves(&self.game, self, 0);

        for mv in move_list.vec() {
            let mut game_clone = self.game.clone();
            let move_result = game_clone.make_move(mv, &self.attack_tables);

            if move_result.is_err() {
                continue;
            }

            let mut nodes = 0;

            let evaluation = -self.negamax_best_move_search(
                &game_clone,
                -max_evaluation,
                -min_evaluation,
                &mut nodes,
                1,
                depth - 1,
            );

            if evaluation > min_evaluation {
                self.historical_move_score
                    .push(mv, self.game.side_to_move(), depth);

                best_move = Some(mv);
                min_evaluation = evaluation;
            }

            total_nodes += nodes;
        }

        match best_move {
            Some(mv) => {
                println!(
                    "info score cp {} depth {} nodes {}",
                    min_evaluation.0, depth, total_nodes
                );

                Ok(mv.clone())
            }
            None => Err(InputError::InvalidPosition),
        }
    }

    fn negamax_best_move_search(
        &mut self,
        game: &Game,
        mut min_evaluation: Evaluation, // alpha
        max_evaluation: Evaluation,     // beta
        nodes: &mut u64,
        ply: Value,
        mut depth: u8,
    ) -> Evaluation {
        if depth == 0 {
            return self.quiescence_search(game, min_evaluation, max_evaluation, nodes);
        }

        *nodes += 1;

        let king_square = game
            .piece_bitboard(Piece::King, game.side_to_move())
            .get_lsb_square();
        let king_in_check = match king_square {
            Some(king_square) => {
                let attacking_side = game.side_to_move().opponent_side();

                game.is_square_attacked(&self.attack_tables, attacking_side, king_square)
            }
            None => false,
        };

        if king_in_check {
            depth += 1;
        }

        let move_list = MoveList::generate_sorted_moves(game, self, ply);
        let mut no_legal_moves = true;

        for mv in move_list.vec() {
            let mut game_clone = game.clone();
            let move_result = game_clone.make_move(mv, &self.attack_tables);

            if move_result.is_err() {
                continue;
            }

            no_legal_moves = false;

            let evaluation = -self.negamax_best_move_search(
                &game_clone,
                -max_evaluation,
                -min_evaluation,
                nodes,
                ply + 1,
                depth - 1,
            );

            if evaluation >= max_evaluation {
                self.killer_moves.push(mv, ply);

                return max_evaluation;
            }

            if evaluation > min_evaluation {
                self.historical_move_score
                    .push(mv, game.side_to_move(), depth);

                min_evaluation = evaluation;
            }
        }

        if no_legal_moves && king_in_check {
            -Evaluation(CHECKMATE_VALUE - ply)
        } else if no_legal_moves {
            Evaluation(STALEMATE_VALUE)
        } else {
            min_evaluation
        }
    }

    fn quiescence_search(
        &self,
        game: &Game,
        mut min_evaluation: Evaluation,
        max_evaluation: Evaluation,
        nodes: &mut u64,
    ) -> Evaluation {
        *nodes += 1;

        let evaluation = Self::evaluate(game).sided_value(game.side_to_move());

        if evaluation >= max_evaluation {
            return max_evaluation;
        }

        if evaluation > min_evaluation {
            min_evaluation = evaluation;
        }

        let move_list = MoveList::generate_sorted_moves(game, self, 0);

        for mv in move_list.vec() {
            if mv.move_type() != MoveType::Capture && mv.move_type() != MoveType::EnPassant {
                continue;
            }

            let mut game_clone = game.clone();
            let move_result = game_clone.make_move(mv, &self.attack_tables);

            if move_result.is_err() {
                continue;
            }

            let evaluation =
                -self.quiescence_search(&game_clone, -max_evaluation, -min_evaluation, nodes);

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
                Piece::Queen => PositionValue([0; 64]),
                Piece::King => KING_POSITION_VALUE,
            };

            while let Some(square) = bitboard.get_lsb_square() {
                evaluation.add(PIECE_VALUE[piece as usize], side);
                evaluation.add(position_value.value(side, square), side);

                bitboard.pop_bit(square);
            }
        }

        evaluation
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
struct Evaluation(Value);

impl Evaluation {
    fn add(&mut self, value: Value, side: Side) {
        self.0 += value * side.to_value()
    }

    fn sided_value(&self, side: Side) -> Evaluation {
        Self(self.0 * side.to_value())
    }
}

impl Neg for Evaluation {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

struct PositionValue([Value; 64]);

impl PositionValue {
    fn value(&self, side: Side, square: Square) -> Value {
        let sided_square_index = match side {
            Side::White => square as usize,
            Side::Black => square.horizontal_mirror() as usize,
        };

        self.0[sided_square_index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_move_checkmate_white() {
        let mut engine = Engine::initialise();

        engine.load_fen("4k3/8/5K2/8/1Q6/8/8/8 w - - 2 1").unwrap();

        let best_move = engine.find_best_move(5).unwrap();

        assert_eq!(best_move.as_string(), "b4e7");

        let mut engine = Engine::initialise();

        engine.load_fen("4k3/8/5K2/8/1Q6/8/8/8 w - - 2 1").unwrap();

        let best_move = engine.find_best_move(6).unwrap();

        assert_eq!(best_move.as_string(), "b4e7");
    }

    #[test]
    fn one_move_checkmate_black() {
        let mut engine = Engine::initialise();

        engine.load_fen("8/8/8/6Q1/8/2K5/8/3k4 w - - 2 1").unwrap();

        let best_move = engine.find_best_move(5).unwrap();

        assert_eq!(best_move.as_string(), "g5d2");

        let mut engine = Engine::initialise();

        engine.load_fen("8/8/8/6Q1/8/2K5/8/3k4 w - - 2 1").unwrap();

        let best_move = engine.find_best_move(6).unwrap();

        assert_eq!(best_move.as_string(), "g5d2");
    }

    #[test]
    fn stalemate_white() {
        let mut engine = Engine::initialise();

        engine
            .load_fen("Q6K/4b3/6q1/8/8/6pp/6pk/8 w - - 0 1")
            .unwrap();

        let best_move = engine.find_best_move(5).unwrap();

        assert_eq!(best_move.as_string(), "a8g2");

        let mut engine = Engine::initialise();

        engine
            .load_fen("Q6K/4b3/6q1/8/8/6pp/6pk/8 w - - 0 1")
            .unwrap();

        let best_move = engine.find_best_move(6).unwrap();

        assert_eq!(best_move.as_string(), "a8g2");
    }

    #[test]
    fn stalemate_black() {
        let mut engine = Engine::initialise();

        engine
            .load_fen("8/KP6/PP6/8/8/1Q6/3B4/k6q b - - 0 1")
            .unwrap();

        let best_move = engine.find_best_move(5).unwrap();

        assert_eq!(best_move.as_string(), "h1b7");

        let mut engine = Engine::initialise();

        engine
            .load_fen("8/KP6/PP6/8/8/1Q6/3B4/k6q b - - 0 1")
            .unwrap();

        let best_move = engine.find_best_move(6).unwrap();

        assert_eq!(best_move.as_string(), "h1b7");
    }

    #[test]
    fn zugzwang_white() {
        let mut engine = Engine::initialise();

        engine.load_fen("6k1/5R2/6K1/8/8/8/8/8 w - - 0 1").unwrap();

        let best_move = engine.find_best_move(5).unwrap();
        let possible_best_moves = ["f7f6", "f7f5", "f7f4,", "f7f3,", "f7f2", "f7f1"];

        assert!(possible_best_moves.contains(&best_move.as_string().as_str()));

        let mut engine = Engine::initialise();

        engine.load_fen("6k1/5R2/6K1/8/8/8/8/8 w - - 0 1").unwrap();

        let best_move = engine.find_best_move(6).unwrap();
        let possible_best_moves = ["f7f6", "f7f5", "f7f4,", "f7f3,", "f7f2", "f7f1"];

        assert!(possible_best_moves.contains(&best_move.as_string().as_str()));
    }

    #[test]
    fn zugzwang_black() {
        let mut engine = Engine::initialise();

        engine.load_fen("8/8/8/8/8/1k6/2r5/1K6 b - - 0 1").unwrap();

        let best_move = engine.find_best_move(5).unwrap();
        let possible_best_moves = ["c2c3", "c2c4", "c2c5", "c2c6", "c2c7", "c2c8"];

        assert!(possible_best_moves.contains(&best_move.as_string().as_str()));

        let mut engine = Engine::initialise();

        engine.load_fen("8/8/8/8/8/1k6/2r5/1K6 b - - 0 1").unwrap();

        let best_move = engine.find_best_move(6).unwrap();
        let possible_best_moves = ["c2c3", "c2c4", "c2c5", "c2c6", "c2c7", "c2c8"];

        assert!(possible_best_moves.contains(&best_move.as_string().as_str()));
    }
}
