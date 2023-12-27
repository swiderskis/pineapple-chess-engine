use super::{
    game::{Game, Piece, Side, Square},
    moves::{Move, MoveList, MoveType},
    Engine,
};
use crate::engine;
use crate::uci::InputError;
use std::{
    ops::{Add, Neg, Sub},
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

pub type Value = i16;

const SEARCH_TIME_OFFSET_MS: u64 = 50;

const MAX_EVALUATION: Evaluation = Evaluation(Value::MAX);
const CHECKMATE_EVALUATION: Evaluation = Evaluation(Value::MAX - 1);
const STALEMATE_EVALUATION: Evaluation = Evaluation(0);

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

const ASPIRATION_WINDOW_ADJUSTMENT: Value = 50;

const NULL_MOVE_DEPTH_MIN: u8 = 3;
const NULL_MOVE_DEPTH_REDUCTION: u8 = 3;

// LMR = late move reduction
const LMR_MOVES_SEARCHED_MIN: i32 = 4;
const LMR_DEPTH_MIN: u8 = 3;
const LMR_DEPTH_REDUCTION: u8 = 2;

pub struct SearchTiming {
    start_time: Instant,
    max_search_time: Duration,
}

impl Engine {
    pub fn search_best_move(&mut self, depth: u8) -> Result<Move, InputError> {
        let mut evaluation_limits = EvaluationLimits::initialise();
        let mut current_depth = 1;
        let ply = 0;

        let game_clone = self.game.clone();

        while current_depth <= depth {
            self.is_principal_variation = true;

            let evaluation =
                self.negamax_search(&game_clone, evaluation_limits, ply, current_depth);

            let missed_aspiration_window_low = evaluation <= evaluation_limits.min;
            let missed_aspiration_window_high = evaluation >= evaluation_limits.max;

            if missed_aspiration_window_low {
                evaluation_limits.min = -MAX_EVALUATION;
                continue;
            } else if missed_aspiration_window_high {
                evaluation_limits.max = MAX_EVALUATION;
                continue;
            }

            evaluation_limits.min = evaluation - ASPIRATION_WINDOW_ADJUSTMENT;
            evaluation_limits.max = evaluation + ASPIRATION_WINDOW_ADJUSTMENT;

            println!(
                "info score cp {} depth {} nodes {} pv {}",
                evaluation.0,
                current_depth,
                self.nodes_searched,
                self.principal_variation.as_string()
            );

            if self.interrupt_search {
                break;
            }

            current_depth += 1;
        }

        let best_move = match self.principal_variation.table[0][0] {
            Some(mv) => Ok(mv),
            None => Err(InputError::InvalidPosition),
        };

        self.clear_parameters();

        best_move
    }

    pub fn set_search_time(
        &mut self,
        increment: Option<Duration>,
        move_time: Option<Duration>,
        time_left: Option<Duration>,
        moves_to_go: u64,
    ) {
        let increment = match increment {
            Some(increment) => increment,
            None => Duration::from_millis(0),
        };

        let max_search_time = match move_time {
            Some(move_time) => move_time,
            None => match time_left {
                Some(time_left) => {
                    let mut max_search_time = time_left;

                    max_search_time /= moves_to_go as u32;
                    max_search_time += increment;
                    max_search_time -= Duration::from_millis(SEARCH_TIME_OFFSET_MS);

                    max_search_time
                }
                None => return,
            },
        };

        let search_time = SearchTiming {
            start_time: Instant::now(),
            max_search_time,
        };
        self.search_timing = Some(search_time);
    }

    pub fn set_stop_search_receiver(&mut self, stop_search_receiver: Receiver<bool>) {
        self.stop_search_receiver = Some(stop_search_receiver);
    }

    fn negamax_search(
        &mut self,
        game: &Game,
        mut evaluation_limits: EvaluationLimits,
        ply: Value,
        mut depth: u8,
    ) -> Evaluation {
        self.stop_search_check();

        self.principal_variation.length[ply as usize] = ply;

        if ply as usize >= engine::MAX_PLY {
            return Self::evaluate(game).sided_value(game.side_to_move());
        }

        if depth == 0 {
            self.is_principal_variation = false;

            return self.quiescence_search(game, evaluation_limits, ply + 1);
        }

        self.nodes_searched += 1;

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

        let apply_null_move_pruning = depth >= NULL_MOVE_DEPTH_MIN && !king_in_check && ply != 0;

        if apply_null_move_pruning {
            let mut game_clone = game.clone();
            game_clone.make_null_move();

            let evaluation = -self.negamax_search(
                &game_clone,
                evaluation_limits.min_narrowed_bounds(),
                ply + 1,
                depth - NULL_MOVE_DEPTH_REDUCTION,
            );

            if evaluation >= evaluation_limits.max {
                return evaluation_limits.max;
            }
        }

        let move_list = MoveList::generate_sorted_moves(game, self, ply);
        let mut moves_searched = 0;

        self.is_principal_variation = match self.principal_variation.principal_move(ply) {
            Some(principal_move) => {
                self.is_principal_variation && move_list.vec()[0] == principal_move
            }
            None => false,
        };

        for mv in move_list.vec() {
            let mut game_clone = game.clone();
            let move_result = game_clone.make_move(*mv, &self.attack_tables);

            if move_result.is_err() {
                continue;
            }

            let apply_late_move_reduction = moves_searched >= LMR_MOVES_SEARCHED_MIN
                && depth >= LMR_DEPTH_MIN
                && !king_in_check
                && !(mv.move_type() == MoveType::Capture)
                && !(mv.move_type() == MoveType::EnPassant)
                && mv.promoted_piece().is_none();

            let evaluation = if moves_searched == 0 {
                -self.negamax_search(&game_clone, -evaluation_limits, ply + 1, depth - 1)
            } else if apply_late_move_reduction {
                self.late_move_reduction_search(&game_clone, evaluation_limits, ply, depth)
            } else {
                self.candidate_best_move_search(&game_clone, evaluation_limits, ply, depth)
            };

            moves_searched += 1;

            if self.interrupt_search {
                return Evaluation(0);
            }

            if evaluation >= evaluation_limits.max {
                self.killer_moves.push(*mv, ply);

                return evaluation_limits.max;
            }

            if evaluation > evaluation_limits.min {
                self.principal_variation.write_move(*mv, ply);
                self.historic_move_score
                    .push(*mv, game_clone.side_to_move(), depth);

                evaluation_limits.min = evaluation;
            }
        }

        if moves_searched == 0 && king_in_check {
            -CHECKMATE_EVALUATION + ply
        } else if moves_searched == 0 {
            STALEMATE_EVALUATION
        } else {
            evaluation_limits.min
        }
    }

    fn quiescence_search(
        &mut self,
        game: &Game,
        mut evaluation_limits: EvaluationLimits,
        ply: Value,
    ) -> Evaluation {
        self.stop_search_check();

        self.nodes_searched += 1;

        let evaluation = Self::evaluate(game).sided_value(game.side_to_move());

        if evaluation >= evaluation_limits.max {
            return evaluation_limits.max;
        }

        if evaluation > evaluation_limits.min {
            evaluation_limits.min = evaluation;
        }

        let move_list = MoveList::generate_sorted_moves(game, self, ply);

        for mv in move_list.vec() {
            if mv.move_type() != MoveType::Capture && mv.move_type() != MoveType::EnPassant {
                continue;
            }

            let mut game_clone = game.clone();
            let move_result = game_clone.make_move(*mv, &self.attack_tables);

            if move_result.is_err() {
                continue;
            }

            let evaluation = -self.quiescence_search(&game_clone, -evaluation_limits, ply + 1);

            if evaluation >= evaluation_limits.max {
                return evaluation_limits.max;
            }

            if evaluation > evaluation_limits.min {
                evaluation_limits.min = evaluation;
            }
        }

        evaluation_limits.min
    }

    fn late_move_reduction_search(
        &mut self,
        game: &Game,
        evaluation_limits: EvaluationLimits,
        ply: Value,
        depth: u8,
    ) -> Evaluation {
        let evaluation = -self.negamax_search(
            game,
            evaluation_limits.max_narrowed_bounds(),
            ply + 1,
            depth - LMR_DEPTH_REDUCTION,
        );

        if evaluation > evaluation_limits.min {
            self.candidate_best_move_search(game, evaluation_limits, ply, depth)
        } else {
            evaluation
        }
    }

    fn candidate_best_move_search(
        &mut self,
        game: &Game,
        evaluation_limits: EvaluationLimits,
        ply: Value,
        depth: u8,
    ) -> Evaluation {
        let evaluation = -self.negamax_search(
            game,
            evaluation_limits.max_narrowed_bounds(),
            ply + 1,
            depth - 1,
        );

        if evaluation > evaluation_limits.min && evaluation < evaluation_limits.max {
            -self.negamax_search(game, -evaluation_limits, ply + 1, depth - 1)
        } else {
            evaluation
        }
    }

    fn stop_search_check(&mut self) {
        if self.nodes_searched & 2047 != 0 || self.interrupt_search {
            return;
        }

        let stop_search_received = match &self.stop_search_receiver {
            Some(receiver) => receiver.try_recv().unwrap_or(self.interrupt_search),
            None => false,
        };

        let max_evaluation_time_exceeded = match &self.search_timing {
            Some(evaluation_time) => {
                evaluation_time.start_time.elapsed() > evaluation_time.max_search_time
            }
            None => false,
        };

        self.interrupt_search = stop_search_received || max_evaluation_time_exceeded
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
                evaluation.sided_add(PIECE_VALUE[piece as usize], side);
                evaluation.sided_add(position_value.value(side, square), side);

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

    pub fn principal_move(&self, ply: Value) -> Option<Move> {
        self.table[0][ply as usize]
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
        self.table[0]
            .iter()
            .flatten()
            .map(|mv| mv.as_string())
            .collect::<Vec<String>>()
            .join(" ")
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
            min: -MAX_EVALUATION,
            max: MAX_EVALUATION,
        }
    }

    fn max_narrowed_bounds(self) -> Self {
        Self {
            min: -self.min - 1,
            max: -self.min,
        }
    }

    fn min_narrowed_bounds(self) -> Self {
        Self {
            min: -self.max,
            max: -self.max + 1,
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
    fn sided_add(&mut self, value: Value, side: Side) {
        self.0 += value * side.to_value()
    }

    fn sided_value(self, side: Side) -> Evaluation {
        Self(self.0 * side.to_value())
    }
}

impl Add<Value> for Evaluation {
    type Output = Self;

    fn add(self, rhs: Value) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Neg for Evaluation {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl Sub<Value> for Evaluation {
    type Output = Self;

    fn sub(self, rhs: Value) -> Self::Output {
        Self(self.0 - rhs)
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

        let best_move = engine.search_best_move(5).unwrap();

        assert_eq!(best_move.as_string(), "b4e7");

        let mut engine = Engine::initialise();

        let fen = vec!["4k3/8/5K2/8/1Q6/8/8/8", "w", "-", "-", "0", "1"];

        engine.load_fen(&fen).unwrap();

        let best_move = engine.search_best_move(6).unwrap();

        assert_eq!(best_move.as_string(), "b4e7");
    }

    #[test]
    fn one_move_checkmate_black() {
        let mut engine = Engine::initialise();

        let fen = vec!["8/8/8/6Q1/8/2K5/8/3k4", "w", "-", "-", "0", "1"];

        engine.load_fen(&fen).unwrap();

        let best_move = engine.search_best_move(5).unwrap();

        assert_eq!(best_move.as_string(), "g5d2");

        let mut engine = Engine::initialise();

        let fen = vec!["8/8/8/6Q1/8/2K5/8/3k4", "w", "-", "-", "0", "1"];

        engine.load_fen(&fen).unwrap();

        let best_move = engine.search_best_move(6).unwrap();

        assert_eq!(best_move.as_string(), "g5d2");
    }

    #[test]
    fn stalemate_white() {
        let mut engine = Engine::initialise();

        let fen = vec!["Q6K/4b3/6q1/8/8/6pp/6pk/8", "w", "-", "-", "0", "1"];

        engine.load_fen(&fen).unwrap();

        let best_move = engine.search_best_move(5).unwrap();

        assert_eq!(best_move.as_string(), "a8g2");

        let mut engine = Engine::initialise();

        let fen = vec!["Q6K/4b3/6q1/8/8/6pp/6pk/8", "w", "-", "-", "0", "1"];

        engine.load_fen(&fen).unwrap();

        let best_move = engine.search_best_move(6).unwrap();

        assert_eq!(best_move.as_string(), "a8g2");
    }

    #[test]
    fn stalemate_black() {
        let mut engine = Engine::initialise();

        let fen = vec!["8/KP6/PP6/8/8/1Q6/3B4/k6q", "b", "-", "-", "0", "1"];

        engine.load_fen(&fen).unwrap();

        let best_move = engine.search_best_move(5).unwrap();

        assert_eq!(best_move.as_string(), "h1b7");

        let mut engine = Engine::initialise();

        let fen = vec!["8/KP6/PP6/8/8/1Q6/3B4/k6q", "b", "-", "-", "0", "1"];

        engine.load_fen(&fen).unwrap();

        let best_move = engine.search_best_move(6).unwrap();

        assert_eq!(best_move.as_string(), "h1b7");
    }

    #[test]
    fn zugzwang_white() {
        let mut engine = Engine::initialise();

        let fen = vec!["6k1/5R2/6K1/8/8/8/8/8", "w", "-", "-", "0", "1"];

        engine.load_fen(&fen).unwrap();

        let best_move = engine.search_best_move(5).unwrap();
        let possible_best_moves = ["f7f6", "f7f5", "f7f4,", "f7f3,", "f7f2", "f7f1"];

        println!("{}", best_move.as_string());
        assert!(possible_best_moves.contains(&best_move.as_string().as_str()));

        let mut engine = Engine::initialise();

        let fen = vec!["6k1/5R2/6K1/8/8/8/8/8", "w", "-", "-", "0", "1"];

        engine.load_fen(&fen).unwrap();

        let best_move = engine.search_best_move(6).unwrap();
        let possible_best_moves = ["f7f6", "f7f5", "f7f4,", "f7f3,", "f7f2", "f7f1"];

        println!("{}", best_move.as_string());
        assert!(possible_best_moves.contains(&best_move.as_string().as_str()));
    }

    #[test]
    fn zugzwang_black() {
        let mut engine = Engine::initialise();

        let fen = vec!["8/8/8/8/8/1k6/2r5/1K6", "b", "-", "-", "0", "1"];

        engine.load_fen(&fen).unwrap();

        let best_move = engine.search_best_move(5).unwrap();
        let possible_best_moves = ["c2c3", "c2c4", "c2c5", "c2c6", "c2c7", "c2c8"];

        println!("{}", best_move.as_string());
        assert!(possible_best_moves.contains(&best_move.as_string().as_str()));

        let mut engine = Engine::initialise();

        let fen = vec!["8/8/8/8/8/1k6/2r5/1K6", "b", "-", "-", "0", "1"];

        engine.load_fen(&fen).unwrap();

        let best_move = engine.search_best_move(6).unwrap();
        let possible_best_moves = ["c2c3", "c2c4", "c2c5", "c2c6", "c2c7", "c2c8"];

        println!("{}", best_move.as_string());
        assert!(possible_best_moves.contains(&best_move.as_string().as_str()));
    }
}
