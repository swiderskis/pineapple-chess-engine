use super::{
    evaluation::Value,
    game::{Game, Piece, Side},
    moves::{Move, MoveList, MoveType},
    Engine,
};
use std::cmp::Reverse;

type Score = u16;

const KILLER_MOVE_ARRAY_SIZE: usize = 2;
const MAX_PLY: usize = 64;

// MVV = most valuable victim
// LVA = least valuable attacker
// Score obtained by indexing to array as such: [attacker][victim]
const MVV_LVA_SCORE: [[Score; 6]; 6] = [
    [10500, 20500, 30500, 40500, 50500, 0],
    [10400, 20400, 30400, 40400, 50400, 0],
    [10300, 20300, 30300, 40300, 50300, 0],
    [10200, 20200, 30200, 40200, 50200, 0],
    [10100, 20100, 30100, 40100, 50100, 0],
    [10000, 20000, 30000, 40000, 50000, 0],
];
const KILLER_MOVE_SCORE: [Score; KILLER_MOVE_ARRAY_SIZE] = [9000, 8000];

pub struct KillerMoves([[Option<Move>; KILLER_MOVE_ARRAY_SIZE]; MAX_PLY]);

impl KillerMoves {
    pub fn initialise() -> Self {
        let empty_killer_moves_array = [(); KILLER_MOVE_ARRAY_SIZE].map(|_| None);

        Self([(); MAX_PLY].map(|_| empty_killer_moves_array.clone()))
    }

    pub fn push(&mut self, mv: &Move, ply: Value) {
        self.0[ply as usize][1] = self.0[ply as usize][0].clone();
        self.0[ply as usize][0] = Some(mv.clone());
    }

    fn score_move(&self, mv: &Move, ply: Value) -> Option<Score> {
        for (index, killer_move) in self.0[ply as usize].iter().flatten().enumerate() {
            if killer_move == mv {
                return Some(KILLER_MOVE_SCORE[index]);
            }
        }

        None
    }
}

pub struct HistoricMoveScore([[[Score; 64]; 6]; 2]);

impl HistoricMoveScore {
    pub fn initialise() -> Self {
        Self([[[0; 64]; 6]; 2])
    }

    pub fn push(&mut self, mv: &Move, side: Side, depth: u8) {
        let piece = mv.piece();
        let target_square = mv.target_square();

        self.0[side as usize][piece as usize][target_square as usize] += depth as Score;
    }

    fn score_move(&self, mv: &Move, side: Side) -> Score {
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
    fn score(&self, game: &Game, engine: &Engine, ply: Value) -> Score {
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
