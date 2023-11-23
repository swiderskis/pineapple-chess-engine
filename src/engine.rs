mod attack_tables;
mod game;
mod moves;

use self::game::Game;

const TRICKY_POSITION: &str =
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

pub fn position() {
    let mut game = Game::initialise(TRICKY_POSITION);

    game::_perft_test(&mut game, 5);
}
