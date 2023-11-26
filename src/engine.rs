mod attack_tables;
mod game;
mod moves;

use self::{
    game::{Game, Piece, Square},
    moves::{Move, MoveFlag, MoveList, MoveSearchParams},
};
use crate::uci::InputError;
use std::str::FromStr;
use strum::ParseError;

#[derive(Clone)]
pub struct Engine {
    game: Game,
    move_list: MoveList,
}

impl Engine {
    pub fn initialise() -> Self {
        let game = Game::initialise();
        let move_list = MoveList::generate_moves(&game);

        Self { game, move_list }
    }

    pub fn load_fen(&mut self, fen: &str) -> Result<(), InputError> {
        self.game.load_fen(fen)?;
        self.move_list = MoveList::generate_moves(&self.game);

        Ok(())
    }

    pub fn make_move(&mut self, mv: &Move) -> Result<(), InputError> {
        self.game.make_move(mv, MoveFlag::All)?;
        self.move_list = MoveList::generate_moves(&self.game);

        Ok(())
    }

    pub fn find_move_from_string(&self, move_string: &str) -> Result<Move, InputError> {
        if let Ok(move_search_params) = Self::parse_move_string(move_string) {
            let mv = self.move_list.find_move(move_search_params)?;

            return Ok(mv);
        }

        Err(InputError::InvalidMoveString(move_string.to_string()))
    }

    fn parse_move_string(move_string: &str) -> Result<MoveSearchParams, ParseError> {
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

        let move_search_params =
            MoveSearchParams::new(source_square, target_square, promoted_piece);

        Ok(move_search_params)
    }
}
