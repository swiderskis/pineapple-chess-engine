use super::{
    evaluation::Value,
    game::{Game, Piece, Side},
    moves::{Move, MoveList, MoveType},
    Engine,
};
use crate::engine;
use std::cmp::Reverse;

type Score = u16;

const PIECE_TYPES: usize = 6;
const SIDE_COUNT: usize = 2;

const KILLER_MOVE_ARRAY_SIZE: usize = 2;

const PRINCIPAL_MOVE_SCORE: Score = 20000;
// MVV = most valuable victim
// LVA = least valuable attacker
// Score obtained by indexing to array as such: [attacker][victim]
const MVV_LVA_SCORE: [[Score; PIECE_TYPES]; PIECE_TYPES] = [
    [10105, 10205, 10305, 10405, 10505, 0],
    [10104, 10204, 10304, 10404, 10504, 0],
    [10103, 10203, 10303, 10403, 10503, 0],
    [10102, 10202, 10302, 10402, 10502, 0],
    [10101, 10201, 10301, 10401, 10501, 0],
    [10100, 10200, 10300, 10400, 10500, 0],
];
const KILLER_MOVE_SCORE: [Score; KILLER_MOVE_ARRAY_SIZE] = [9000, 8000];

pub struct KillerMoves([[Option<Move>; KILLER_MOVE_ARRAY_SIZE]; engine::MAX_PLY]);

impl KillerMoves {
    pub fn initialise() -> Self {
        Self([[None; KILLER_MOVE_ARRAY_SIZE]; engine::MAX_PLY])
    }

    pub fn push(&mut self, mv: Move, ply: Value) {
        if mv.move_type() == MoveType::Capture || mv.move_type() == MoveType::EnPassant {
            return;
        }

        self.0[ply as usize][1] = self.0[ply as usize][0];
        self.0[ply as usize][0] = Some(mv);
    }

    fn score_move(&self, mv: Move, ply: Value) -> Option<Score> {
        for (index, killer_move) in self.0[ply as usize].iter().flatten().enumerate() {
            if *killer_move == mv {
                return Some(KILLER_MOVE_SCORE[index]);
            }
        }

        None
    }
}

pub struct HistoricMoveScore([[[Score; 64]; PIECE_TYPES]; SIDE_COUNT]);

impl HistoricMoveScore {
    pub fn initialise() -> Self {
        Self([[[0; 64]; PIECE_TYPES]; SIDE_COUNT])
    }

    pub fn push(&mut self, mv: Move, side: Side, depth: u8) {
        if mv.move_type() == MoveType::Capture || mv.move_type() == MoveType::EnPassant {
            return;
        }

        let piece = mv.piece();
        let target_square = mv.target_square();

        self.0[side as usize][piece as usize][target_square as usize] += (depth * depth) as Score;
    }

    fn score_move(&self, mv: Move, side: Side) -> Score {
        let piece = mv.piece();
        let target_square = mv.target_square();

        self.0[side as usize][piece as usize][target_square as usize]
    }
}

impl MoveList {
    pub fn generate_sorted_moves(game: &Game, engine: &Engine, ply: Value) -> Self {
        let mut move_list = Self::generate_moves(game, &engine.attack_tables);

        move_list
            .mut_vec()
            .sort_by_key(|mv| Reverse(mv.score(game, engine, ply)));

        move_list
    }
}

impl Move {
    fn score(self, game: &Game, engine: &Engine, ply: Value) -> Score {
        if let Some(principal_move) = engine.principal_variation.principal_move(ply) {
            if engine.is_principal_variation && principal_move == self {
                return PRINCIPAL_MOVE_SCORE;
            }
        }

        match self.move_type() {
            MoveType::Capture => match game.piece_at_square(self.target_square()) {
                Some((victim, _)) => {
                    let attacker = self.piece();

                    MVV_LVA_SCORE[attacker as usize][victim as usize]
                }
                None => 0,
            },
            MoveType::EnPassant => MVV_LVA_SCORE[Piece::Pawn as usize][Piece::Pawn as usize],
            _ => engine.killer_moves.score_move(self, ply).unwrap_or(
                engine
                    .historic_move_score
                    .score_move(self, game.side_to_move()),
            ),
        }
    }
}
