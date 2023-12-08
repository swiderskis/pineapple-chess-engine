mod attack_tables;
mod evaluation;
mod game;
mod moves;

use self::{
    attack_tables::AttackTables,
    game::{Game, Piece, Square},
    moves::{Move, MoveList, MoveSearch},
};
use crate::uci::InputError;
use std::str::FromStr;
use strum::ParseError;

pub struct Engine {
    game: Game,
    move_list: MoveList,
    attack_tables: AttackTables,
}

impl Engine {
    pub fn initialise() -> Self {
        Self {
            game: Game::initialise(),
            move_list: MoveList::new(),
            attack_tables: AttackTables::initialise(),
        }
    }

    pub fn load_fen(&mut self, fen: &str) -> Result<(), InputError> {
        self.game.load_fen(fen)?;
        self.move_list = MoveList::generate_sorted_moves(&self.game, &self.attack_tables);

        Ok(())
    }

    pub fn make_move(&mut self, mv: &Move) -> Result<(), InputError> {
        self.game.make_move(mv, &self.attack_tables)?;
        self.move_list = MoveList::generate_sorted_moves(&self.game, &self.attack_tables);

        Ok(())
    }

    pub fn find_move_from_string(&self, move_string: &str) -> Result<Move, InputError> {
        match Self::parse_move_string(move_string) {
            Ok(move_search) => {
                let mv = self.move_list.find_move(move_search)?;

                Ok(mv)
            }
            Err(_) => Err(InputError::InvalidMoveString),
        }
    }

    fn parse_move_string(move_string: &str) -> Result<MoveSearch, ParseError> {
        let (source_square_string, remaining_move_string) = move_string.split_at(2);
        let (target_square_string, promoted_piece_string) = remaining_move_string.split_at(2);

        let source_square = Square::from_str(source_square_string.to_uppercase().as_str())?;
        let target_square = Square::from_str(target_square_string.to_uppercase().as_str())?;

        let promoted_piece = if let Some(promoted_piece_char) = promoted_piece_string.chars().nth(0)
        {
            Some(Piece::from_char(promoted_piece_char)?)
        } else {
            None
        };

        let move_search = MoveSearch::new(source_square, target_square, promoted_piece);

        Ok(move_search)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_start_position() {
        let mut engine = Engine::initialise();

        engine.load_fen("startpos").unwrap();

        assert_eq!(engine.move_list._length(), 20);
    }

    #[test]
    fn load_tricky_position() {
        let mut engine = Engine::initialise();

        engine
            .load_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap();

        assert_eq!(engine.move_list._length(), 48);
    }

    #[test]
    fn start_position_moves() {
        let mut engine = Engine::initialise();

        engine.load_fen("startpos").unwrap();

        let mv = engine.find_move_from_string("e2e4").unwrap();

        engine.make_move(&mv).unwrap();

        let mv = engine.find_move_from_string("e7e5").unwrap();

        engine.make_move(&mv).unwrap();

        let mv = engine.find_move_from_string("g1f3").unwrap();

        engine.make_move(&mv).unwrap();
    }

    #[test]
    fn killer_position_moves() {
        let mut engine = Engine::initialise();

        engine
            .load_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap();

        let mv = engine.find_move_from_string("d5e6").unwrap();

        engine.make_move(&mv).unwrap();

        let mv = engine.find_move_from_string("a6e2").unwrap();

        engine.make_move(&mv).unwrap();

        let mv = engine.find_move_from_string("c3e2").unwrap();

        engine.make_move(&mv).unwrap();
    }

    #[test]
    fn parse_move() {
        let move_string = "e2e4";

        let move_search = Engine::parse_move_string(move_string).unwrap();

        let desired_move_search = MoveSearch::new(Square::E2, Square::E4, None);

        assert_eq!(move_search, desired_move_search);

        let move_string = "e7e8q";

        let move_search = Engine::parse_move_string(move_string).unwrap();

        let desired_move_search = MoveSearch::new(Square::E7, Square::E8, Some(Piece::Queen));

        assert_eq!(move_search, desired_move_search);

        let move_string = "e2e1r";

        let move_search = Engine::parse_move_string(move_string).unwrap();

        let desired_move_search = MoveSearch::new(Square::E2, Square::E1, Some(Piece::Rook));

        assert_eq!(move_search, desired_move_search);

        let move_string = "d7d8b";

        let move_search = Engine::parse_move_string(move_string).unwrap();

        let desired_move_search = MoveSearch::new(Square::D7, Square::D8, Some(Piece::Bishop));

        assert_eq!(move_search, desired_move_search);

        let move_string = "d2d1n";

        let move_search = Engine::parse_move_string(move_string).unwrap();

        let desired_move_search = MoveSearch::new(Square::D2, Square::D1, Some(Piece::Knight));

        assert_eq!(move_search, desired_move_search);
    }
}
