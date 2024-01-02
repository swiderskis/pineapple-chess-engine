mod attack_tables;
mod game;
mod moves;
mod search;
mod zobrist_hashes;

use self::{attack_tables::AttackTables, game::Game, moves::MoveList, search::SearchParameters};
use crate::uci::InputError;

pub const MAX_PLY: usize = 64;

pub struct Engine {
    game: Game,
    attack_tables: AttackTables,
    search_parameters: SearchParameters,
}

impl Engine {
    pub fn initialise() -> Self {
        Self {
            game: Game::initialise(),
            attack_tables: AttackTables::initialise(),
            search_parameters: SearchParameters::initialise(),
        }
    }

    pub fn load_fen(&mut self, fen: &[&str]) -> Result<(), InputError> {
        self.game.load_fen(fen)?;

        Ok(())
    }

    pub fn make_move(&mut self, move_string: &str) -> Result<(), InputError> {
        let move_list = MoveList::generate_moves(&self.game, &self.attack_tables);
        let mv = move_list.find_move_from_string(move_string)?;
        self.game.make_move(mv, &self.attack_tables)?;

        Ok(())
    }

    pub fn reset_game(&mut self) {
        self.game = Game::initialise();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_start_position() {
        let mut engine = Engine::initialise();
        let fen = vec!["startpos"];
        engine.load_fen(&fen).unwrap();

        let move_list = MoveList::generate_moves(&engine.game, &engine.attack_tables);

        assert_eq!(move_list._length(), 20);
    }

    #[test]
    fn load_tricky_position() {
        let mut engine = Engine::initialise();
        let fen = vec![
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R",
            "w",
            "KQkq",
            "-",
            "0",
            "1",
        ];
        engine.load_fen(&fen).unwrap();

        let move_list = MoveList::generate_moves(&engine.game, &engine.attack_tables);

        assert_eq!(move_list._length(), 48);
    }

    #[test]
    fn start_position_moves() {
        let mut engine = Engine::initialise();
        let fen = vec!["startpos"];
        engine.load_fen(&fen).unwrap();
        engine.make_move("e2e4").unwrap();
        engine.make_move("e7e5").unwrap();
        engine.make_move("g1f3").unwrap();
    }

    #[test]
    fn killer_position_moves() {
        let mut engine = Engine::initialise();
        let fen = vec![
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R",
            "w",
            "KQkq",
            "-",
            "0",
            "1",
        ];
        engine.load_fen(&fen).unwrap();
        engine.make_move("d5e6").unwrap();
        engine.make_move("a6e2").unwrap();
        engine.make_move("c3e2").unwrap();
    }
}
