use super::{
    game::{Game, Piece, Side, Square},
    moves::{Move, MoveList, MoveType},
    Engine,
};
use crate::uci::InputError;
use std::ops::Neg;

const MAX_EVALUATION_VALUE: i32 = i32::MAX;
const CHECKMATE_VALUE: i32 = i32::MAX - 1;
const STALEMATE_VALUE: i32 = 0;

// Piece value obtained by indexing into array using Piece enum
const PIECE_VALUE: [i32; 6] = [100, 300, 350, 500, 900, 10_000];

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

#[derive(Clone, Copy, PartialEq, PartialOrd)]
struct Evaluation(i32);

impl Evaluation {
    fn add(&mut self, value: i32, side: Side) {
        self.0 += value * side as i32
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
    pub fn find_best_move(&self, depth: u8) -> Result<Move, InputError> {
        let mut min_evaluation = Evaluation(-MAX_EVALUATION_VALUE);
        let max_evaluation = Evaluation(MAX_EVALUATION_VALUE);

        let mut best_move = None;

        let current_ply = 0;
        let mut total_nodes = 0;

        for mv in self.move_list.move_list().iter().flatten() {
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
                current_ply + 1,
                depth - 1,
                &mut nodes,
            );

            if evaluation > min_evaluation {
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
        &self,
        game: &Game,
        mut min_evaluation: Evaluation, // alpha
        max_evaluation: Evaluation,     // beta
        current_ply: u8,
        depth: u8,
        nodes: &mut u64,
    ) -> Evaluation {
        if depth == 0 {
            return self.quiescence_search(game, min_evaluation, max_evaluation, nodes);
        }

        *nodes += 1;

        let move_list = MoveList::generate_moves(game, &self.attack_tables);

        let mut no_legal_moves = true;

        for mv in move_list.move_list().iter().flatten() {
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
                current_ply + 1,
                depth - 1,
                nodes,
            );

            if evaluation >= max_evaluation {
                return max_evaluation;
            }

            if evaluation > min_evaluation {
                min_evaluation = evaluation;
            }
        }

        if no_legal_moves {
            let king_square = game
                .piece_bitboard(Piece::King, game.side_to_move())
                .get_lsb_square();

            if let Some(king_square) = king_square {
                let attacking_side = game.side_to_move().opponent_side();
                let king_in_check =
                    game.is_square_attacked(&self.attack_tables, attacking_side, king_square);

                if king_in_check {
                    return -Evaluation(CHECKMATE_VALUE - current_ply as i32);
                } else {
                    return Evaluation(STALEMATE_VALUE);
                }
            }
        }

        min_evaluation
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

        let move_list = MoveList::generate_moves(game, &self.attack_tables);

        for mv in move_list.move_list().iter().flatten() {
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

struct PositionValue([i32; 64]);

impl PositionValue {
    fn value(&self, side: Side, square: Square) -> i32 {
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
