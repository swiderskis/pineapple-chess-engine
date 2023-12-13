use crate::engine::Engine;
use std::{fmt::Display, str::FromStr};

const FEN_MOVES_STARTING_INDEX: usize = 7;
const STARTPOS_MOVES_STARTING_INDEX: usize = 1;

struct Input<'a> {
    command: &'a str,
    arguments: Vec<&'a str>,
}

impl<'a> Input<'a> {
    fn new(input: &'a str) -> Self {
        let input: Vec<&str> = input.split_whitespace().collect();

        let command = match input.first() {
            Some(command) => command,
            None => "",
        };
        let arguments = match input.get(1) {
            Some(_) => input[1..].to_vec(),
            None => Vec::new(),
        };

        Self { command, arguments }
    }
}

pub fn engine() {
    let mut engine = Engine::initialise();

    loop {
        let mut input = String::new();

        if std::io::stdin().read_line(&mut input).is_err() {
            println!("Failed to parse input");

            continue;
        }

        let input = Input::new(&input);

        match input.command {
            "uci" => uci(),
            "isready" => println!("readyok"),
            "ucinewgame" => handle_command(ucinewgame, &mut engine, input.arguments),
            "position" => handle_command(position, &mut engine, input.arguments),
            "go" => handle_command(go, &mut engine, input.arguments),
            "quit" => break,
            "" => {}
            _ => println!("Unknown command"),
        }
    }
}

fn uci() {
    println!("id name Pineapple");
    println!("id author Sebastian S.");
    println!("uciok");
}

fn ucinewgame(engine: &mut Engine, _arguments: Vec<&str>) -> Result<(), InputError> {
    engine.reset();

    let fen = vec!["startpos"];
    engine.load_fen(&fen)?;

    Ok(())
}

fn position(engine: &mut Engine, arguments: Vec<&str>) -> Result<(), InputError> {
    if arguments.is_empty() {
        return Err(InputError::InvalidPositionArguments);
    }

    let moves_starting_index = match arguments[0] {
        "fen" => {
            if arguments.get(FEN_MOVES_STARTING_INDEX - 1).is_none() {
                return Err(InputError::InvalidPositionArguments);
            }

            let fen: Vec<&str> = arguments
                .clone()
                .drain(1..FEN_MOVES_STARTING_INDEX)
                .collect();
            engine.load_fen(&fen)?;

            FEN_MOVES_STARTING_INDEX
        }
        "startpos" => {
            engine.load_fen(&arguments)?;

            STARTPOS_MOVES_STARTING_INDEX
        }
        _ => return Err(InputError::InvalidPositionArguments),
    };

    match arguments.get(moves_starting_index) {
        Some(argument) => {
            if *argument != "moves" {
                return Err(InputError::InvalidPositionArguments);
            }
        }
        None => return Ok(()),
    }

    for move_string in arguments[moves_starting_index + 1..].iter() {
        make_move_from_string(engine, move_string)?;
    }

    Ok(())
}

fn go(engine: &mut Engine, arguments: Vec<&str>) -> Result<(), InputError> {
    let mut depth = 1;

    for (index, argument) in arguments.iter().enumerate() {
        match *argument {
            "depth" => {
                depth = get_argument_value(&arguments, index, InputError::InvalidGoArguments)?;

                if depth == 0 {
                    return Err(InputError::InvalidGoArguments);
                }
            }
            _ => continue,
        }
    }

    let best_move = engine.find_best_move(depth)?.as_string();

    println!("bestmove {}", best_move);

    Ok(())
}

fn make_move_from_string(engine: &mut Engine, move_string: &str) -> Result<(), InputError> {
    engine.make_move(move_string)?;

    Ok(())
}

fn handle_command<F>(command_fn: F, engine: &mut Engine, arguments: Vec<&str>)
where
    F: Fn(&mut Engine, Vec<&str>) -> Result<(), InputError>,
{
    let result = command_fn(engine, arguments);

    if let Err(error) = result {
        println!("{}", error);
    }
}

fn get_argument_value<T: FromStr>(
    arguments: &[&str],
    index: usize,
    error: InputError,
) -> Result<T, InputError> {
    match arguments.get(index + 1) {
        Some(value_str) => match value_str.parse() {
            Ok(value) => Ok(value),
            Err(_) => Err(error),
        },
        None => Err(error),
    }
}

#[derive(Debug)]
pub enum InputError {
    IllegalMove,
    InvalidFen(FenError),
    InvalidGoArguments,
    InvalidMoveString,
    InvalidPosition,
    InvalidPositionArguments,
}

impl Display for InputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IllegalMove => write!(f, "Attempted to play an illegal move"),
            Self::InvalidFen(error) => write!(f, "Failed to parse FEN: {}", error),
            Self::InvalidGoArguments => write!(f, "Invalid go command arguments"),
            Self::InvalidMoveString => write!(f, "Failed to parse move string"),
            Self::InvalidPosition => write!(f, "Invalid board position"),
            Self::InvalidPositionArguments => write!(f, "Invalid position command arguments"),
        }
    }
}

#[derive(Debug)]
pub enum FenError {
    BoardPosition,
    SideToMove,
    CastlingRights,
    EnPassantSquare,
    ParseHalfmoveClock,
    InvalidHalfmoveClock,
}

impl Display for FenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BoardPosition => write!(f, "unable to parse board position"),
            Self::SideToMove => write!(f, "unable to parse side to move"),
            Self::CastlingRights => write!(f, "unable to parse castling rights"),
            Self::EnPassantSquare => write!(f, "unable to parse en passant square"),
            Self::ParseHalfmoveClock => write!(f, "unable to parse ply"),
            Self::InvalidHalfmoveClock => write!(f, "invalid halfmove value provided"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_position() {
        let mut engine = Engine::initialise();

        let input = "position startpos moves e2e4 e7e5 g1f3";
        let input = Input::new(input);

        position(&mut engine, input.arguments).unwrap();
    }

    #[test]
    fn killer_position() {
        let mut engine = Engine::initialise();

        let input =
            "position fen r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1 \
            moves d5e6 a6e2 c3e2";
        let input = Input::new(input);

        position(&mut engine, input.arguments).unwrap();
    }
}
