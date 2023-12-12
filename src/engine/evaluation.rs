use super::{
    game::{Game, Piece, Side, Square},
    moves::{Move, MoveList, MoveType},
    Engine,
};
use crate::engine;
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
        let mut evaluation_limits = EvaluationLimits::initialise();
        let ply = 0;

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

        let move_list = MoveList::generate_sorted_moves(&self.game, self, 0);

        for mv in move_list.vec() {
            let mut game_clone = self.game.clone();
            let move_result = game_clone.make_move(*mv, &self.attack_tables);

            if move_result.is_err() {
                continue;
            }

            let mut nodes = 0;

            let evaluation = -self.negamax_best_move_search(
                &game_clone,
                -evaluation_limits,
                &mut nodes,
                ply + 1,
                depth - 1,
            );

            if evaluation > evaluation_limits.min {
                self.principal_variation.write_move(*mv, ply);
                self.historic_move_score
                    .push(*mv, self.game.side_to_move(), depth);

                evaluation_limits.min = evaluation;
            }

            total_nodes += nodes;
        }

        match self.principal_variation.table[0][0] {
            Some(mv) => {
                println!(
                    "info score cp {} depth {} nodes {} pv {}",
                    evaluation_limits.min.0,
                    depth,
                    total_nodes,
                    self.principal_variation.as_string()
                );

                Ok(mv)
            }
            None => Err(InputError::InvalidPosition),
        }
    }

    fn negamax_best_move_search(
        &mut self,
        game: &Game,
        mut evaluation_limits: EvaluationLimits,
        nodes: &mut u64,
        ply: Value,
        mut depth: u8,
    ) -> Evaluation {
        self.principal_variation.length[ply as usize] = ply;

        if depth == 0 {
            return self.quiescence_search(game, evaluation_limits, nodes);
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
            let move_result = game_clone.make_move(*mv, &self.attack_tables);

            if move_result.is_err() {
                continue;
            }

            no_legal_moves = false;

            let evaluation = -self.negamax_best_move_search(
                &game_clone,
                -evaluation_limits,
                nodes,
                ply + 1,
                depth - 1,
            );

            if evaluation >= evaluation_limits.max {
                self.killer_moves.push(*mv, ply);

                return evaluation_limits.max;
            }

            if evaluation > evaluation_limits.min {
                self.principal_variation.write_move(*mv, ply);

                self.historic_move_score
                    .push(*mv, game.side_to_move(), depth);

                evaluation_limits.min = evaluation;
            }
        }

        if no_legal_moves && king_in_check {
            -Evaluation(CHECKMATE_VALUE - ply)
        } else if no_legal_moves {
            Evaluation(STALEMATE_VALUE)
        } else {
            evaluation_limits.min
        }
    }

    fn quiescence_search(
        &self,
        game: &Game,
        mut evaluation_limits: EvaluationLimits,
        nodes: &mut u64,
    ) -> Evaluation {
        *nodes += 1;

        let evaluation = Self::evaluate(game).sided_value(game.side_to_move());

        if evaluation >= evaluation_limits.max {
            return evaluation_limits.max;
        }

        if evaluation > evaluation_limits.min {
            evaluation_limits.min = evaluation;
        }

        let move_list = MoveList::generate_sorted_moves(game, self, 0);

        for mv in move_list.vec() {
            if mv.move_type() != MoveType::Capture && mv.move_type() != MoveType::EnPassant {
                continue;
            }

            let mut game_clone = game.clone();
            let move_result = game_clone.make_move(*mv, &self.attack_tables);

            if move_result.is_err() {
                continue;
            }

            let evaluation = -self.quiescence_search(&game_clone, -evaluation_limits, nodes);

            if evaluation >= evaluation_limits.max {
                return evaluation_limits.max;
            }

            if evaluation > evaluation_limits.min {
                evaluation_limits.min = evaluation;
            }
        }

        evaluation_limits.min
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

pub struct PrincipalVariation {
    table: [[Option<Move>; engine::MAX_PLY]; engine::MAX_PLY],
    length: [Value; engine::MAX_PLY],
}

impl PrincipalVariation {
    pub fn initialise() -> Self {
        Self {
            table: [[None; engine::MAX_PLY]; engine::MAX_PLY],
            length: [0; engine::MAX_PLY],
        }
    }

    fn write_move(&mut self, mv: Move, ply: Value) {
        let ply = ply as usize;

        self.table[ply][ply] = Some(mv);

        for next_ply in (ply + 1)..self.length[ply + 1] as usize {
            self.table[ply][next_ply] = self.table[ply + 1][next_ply];
        }

        self.length[ply] = self.length[ply + 1];
    }

    fn as_string(&self) -> String {
        let mut move_list_string = String::new();

        for mv in self.table[0].iter().flatten() {
            let move_string = mv.as_string() + " ";

            move_list_string += &move_string;
        }

        move_list_string
    }
}

#[derive(Clone, Copy)]
struct EvaluationLimits {
    min: Evaluation, // alpha
    max: Evaluation, // beta
}

impl EvaluationLimits {
    fn initialise() -> Self {
        Self {
            min: -Evaluation(MAX_EVALUATION_VALUE),
            max: Evaluation(MAX_EVALUATION_VALUE),
        }
    }
}

impl Neg for EvaluationLimits {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let new_min = -self.max;
        let new_max = -self.min;

        Self {
            min: new_min,
            max: new_max,
        }
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
struct Evaluation(Value);

impl Evaluation {
    fn add(&mut self, value: Value, side: Side) {
        self.0 += value * side.to_value()
    }

    fn sided_value(self, side: Side) -> Evaluation {
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

        let fen = vec!["4k3/8/5K2/8/1Q6/8/8/8", "w", "-", "-", "0", "1"];

        engine.load_fen(&fen).unwrap();

        let best_move = engine.find_best_move(5).unwrap();

        assert_eq!(best_move.as_string(), "b4e7");

        let mut engine = Engine::initialise();

        let fen = vec!["4k3/8/5K2/8/1Q6/8/8/8", "w", "-", "-", "0", "1"];

        engine.load_fen(&fen).unwrap();

        let best_move = engine.find_best_move(6).unwrap();

        assert_eq!(best_move.as_string(), "b4e7");
    }

    #[test]
    fn one_move_checkmate_black() {
        let mut engine = Engine::initialise();

        let fen = vec!["8/8/8/6Q1/8/2K5/8/3k4", "w", "-", "-", "0", "1"];

        engine.load_fen(&fen).unwrap();

        let best_move = engine.find_best_move(5).unwrap();

        assert_eq!(best_move.as_string(), "g5d2");

        let mut engine = Engine::initialise();

        let fen = vec!["8/8/8/6Q1/8/2K5/8/3k4", "w", "-", "-", "0", "1"];

        engine.load_fen(&fen).unwrap();

        let best_move = engine.find_best_move(6).unwrap();

        assert_eq!(best_move.as_string(), "g5d2");
    }

    #[test]
    fn stalemate_white() {
        let mut engine = Engine::initialise();

        let fen = vec!["Q6K/4b3/6q1/8/8/6pp/6pk/8", "w", "-", "-", "0", "1"];

        engine.load_fen(&fen).unwrap();

        let best_move = engine.find_best_move(5).unwrap();

        assert_eq!(best_move.as_string(), "a8g2");

        let mut engine = Engine::initialise();

        let fen = vec!["Q6K/4b3/6q1/8/8/6pp/6pk/8", "w", "-", "-", "0", "1"];

        engine.load_fen(&fen).unwrap();

        let best_move = engine.find_best_move(6).unwrap();

        assert_eq!(best_move.as_string(), "a8g2");
    }

    #[test]
    fn stalemate_black() {
        let mut engine = Engine::initialise();

        let fen = vec!["8/KP6/PP6/8/8/1Q6/3B4/k6q", "b", "-", "-", "0", "1"];

        engine.load_fen(&fen).unwrap();

        let best_move = engine.find_best_move(5).unwrap();

        assert_eq!(best_move.as_string(), "h1b7");

        let mut engine = Engine::initialise();

        let fen = vec!["8/KP6/PP6/8/8/1Q6/3B4/k6q", "b", "-", "-", "0", "1"];

        engine.load_fen(&fen).unwrap();

        let best_move = engine.find_best_move(6).unwrap();

        assert_eq!(best_move.as_string(), "h1b7");
    }

    #[test]
    fn zugzwang_white() {
        let mut engine = Engine::initialise();

        let fen = vec!["6k1/5R2/6K1/8/8/8/8/8", "w", "-", "-", "0", "1"];

        engine.load_fen(&fen).unwrap();

        let best_move = engine.find_best_move(5).unwrap();
        let possible_best_moves = ["f7f6", "f7f5", "f7f4,", "f7f3,", "f7f2", "f7f1"];

        assert!(possible_best_moves.contains(&best_move.as_string().as_str()));

        let mut engine = Engine::initialise();

        let fen = vec!["6k1/5R2/6K1/8/8/8/8/8", "w", "-", "-", "0", "1"];

        engine.load_fen(&fen).unwrap();

        let best_move = engine.find_best_move(6).unwrap();
        let possible_best_moves = ["f7f6", "f7f5", "f7f4,", "f7f3,", "f7f2", "f7f1"];

        assert!(possible_best_moves.contains(&best_move.as_string().as_str()));
    }

    #[test]
    fn zugzwang_black() {
        let mut engine = Engine::initialise();

        let fen = vec!["8/8/8/8/8/1k6/2r5/1K6", "b", "-", "-", "0", "1"];

        engine.load_fen(&fen).unwrap();

        let best_move = engine.find_best_move(5).unwrap();
        let possible_best_moves = ["c2c3", "c2c4", "c2c5", "c2c6", "c2c7", "c2c8"];

        assert!(possible_best_moves.contains(&best_move.as_string().as_str()));

        let mut engine = Engine::initialise();

        let fen = vec!["8/8/8/8/8/1k6/2r5/1K6", "b", "-", "-", "0", "1"];

        engine.load_fen(&fen).unwrap();

        let best_move = engine.find_best_move(6).unwrap();
        let possible_best_moves = ["c2c3", "c2c4", "c2c5", "c2c6", "c2c7", "c2c8"];

        assert!(possible_best_moves.contains(&best_move.as_string().as_str()));
    }
}
