pub mod move_scoring;

mod evaluation;

pub use self::evaluation::Value;

use self::{
    evaluation::Evaluation,
    move_scoring::{HistoricMoveScore, KillerMoves},
};
use super::{
    attack_tables::AttackTables,
    game::{Game, Piece},
    moves::{Move, MoveList, MoveType},
    Engine,
};
use crate::{
    engine::{self},
    uci::InputError,
};
use std::{
    ops::Neg,
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

const ASPIRATION_WINDOW_ADJUSTMENT: Value = 50;

const NULL_MOVE_DEPTH_MIN: u8 = 3;
const NULL_MOVE_DEPTH_REDUCTION: u8 = 3;

// LMR = late move reduction
const LMR_MOVES_SEARCHED_MIN: i32 = 4;
const LMR_DEPTH_MIN: u8 = 3;
const LMR_DEPTH_REDUCTION: u8 = 2;

const SEARCH_TIME_OFFSET_MS: u64 = 50;

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
                evaluation_limits.min = -evaluation::MAX_EVALUATION;
                continue;
            } else if missed_aspiration_window_high {
                evaluation_limits.max = evaluation::MAX_EVALUATION;
                continue;
            }

            evaluation_limits.min = evaluation - ASPIRATION_WINDOW_ADJUSTMENT;
            evaluation_limits.max = evaluation + ASPIRATION_WINDOW_ADJUSTMENT;

            println!(
                "info score cp {} depth {} nodes {} pv {}",
                evaluation.value(),
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

        self.clear_search_parameters();

        best_move
    }

    pub fn set_search_timing(
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

        let search_timing = SearchTiming {
            start_time: Instant::now(),
            max_search_time,
        };
        self.search_timing = Some(search_timing);
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
                return evaluation::STALEMATE_EVALUATION;
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
            -evaluation::CHECKMATE_EVALUATION + ply
        } else if moves_searched == 0 {
            evaluation::STALEMATE_EVALUATION
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

    fn clear_search_parameters(&mut self) {
        self.principal_variation = PrincipalVariation::initialise();
        self.killer_moves = KillerMoves::initialise();
        self.historic_move_score = HistoricMoveScore::initialise();
        self.is_principal_variation = true;
        self.search_timing = None;
        self.interrupt_search = false;
        self.nodes_searched = 0;
    }
}

pub struct SearchTiming {
    start_time: Instant,
    max_search_time: Duration,
}

#[derive(Clone, Copy)]
pub struct EvaluationLimits {
    min: Evaluation, // alpha
    max: Evaluation, // beta
}

impl EvaluationLimits {
    pub fn initialise() -> Self {
        Self {
            min: -evaluation::MAX_EVALUATION,
            max: evaluation::MAX_EVALUATION,
        }
    }

    pub fn max_narrowed_bounds(self) -> Self {
        Self {
            min: -self.min - 1,
            max: -self.min,
        }
    }

    pub fn min_narrowed_bounds(self) -> Self {
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

fn _perft_test(game: &mut Game, attack_tables: &AttackTables, depth: u8) {
    let mut total_nodes = 0;
    let now = Instant::now();

    let move_list = MoveList::generate_moves(game, attack_tables);

    println!("Move   Nodes   ");

    for mv in move_list.vec() {
        let mut game_clone = game.clone();
        let move_result = game_clone.make_move(*mv, attack_tables);

        if move_result.is_err() {
            continue;
        }

        let mut nodes = 0;

        _perft(&mut game_clone, attack_tables, &mut nodes, depth - 1);

        print!("{:<6}", mv.as_string());
        print!("{:^7}", nodes);
        println!();

        total_nodes += nodes;
    }

    println!();
    println!("Depth: {}", depth);
    println!("Nodes: {}", total_nodes);
    println!("Time taken: {:?}", now.elapsed());
}

fn _perft(game: &mut Game, attack_tables: &AttackTables, nodes: &mut u64, depth: u8) {
    if depth == 0 {
        *nodes += 1;

        return;
    }

    let move_list = MoveList::generate_moves(game, attack_tables);

    for mv in move_list.vec() {
        let mut game_clone = game.clone();
        let move_result = game_clone.make_move(*mv, attack_tables);

        if move_result.is_err() {
            continue;
        }

        _perft(&mut game_clone, attack_tables, nodes, depth - 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perft_start_position() {
        let mut game = Game::initialise();
        let attack_tables = AttackTables::initialise();

        let fen = vec!["startpos"];

        game.load_fen(&fen).unwrap();

        let mut nodes = 0;

        _perft(&mut game, &attack_tables, &mut nodes, 6);

        assert_eq!(nodes, 119_060_324);
    }

    #[test]
    fn perft_tricky_position() {
        let mut game = Game::initialise();
        let attack_tables = AttackTables::initialise();

        let fen = vec![
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R",
            "w",
            "KQkq",
            "-",
            "0",
            "1",
        ];

        game.load_fen(&fen).unwrap();

        let mut nodes = 0;

        _perft(&mut game, &attack_tables, &mut nodes, 5);

        assert_eq!(nodes, 193_690_690);
    }

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
