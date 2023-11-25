use crate::engine::Engine;
use std::{fmt::Display, io};

const _TRICKY_POSITION: &str =
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

struct Input<'a> {
    command: &'a str,
    arguments: Vec<&'a str>,
}

impl<'a> Input<'a> {
    fn new(input_vec: Vec<&'a str>) -> Self {
        Self {
            command: input_vec[0],
            arguments: input_vec[1..].to_vec(),
        }
    }
}

pub fn command() {
    let mut engine = Engine::initialise();

    loop {
        let mut input = String::new();

        match io::stdin().read_line(&mut input) {
            Ok(_) => {}
            Err(error) => {
                println!("Error parsing command: {}", error);
                continue;
            }
        };

        let input: Vec<&str> = input.split_whitespace().collect();
        let input = Input::new(input);

        match input.command.trim() {
            "uci" => uci(),
            "isready" => println!("readyok"),
            "ucinewgame" => continue,
            "position" => match position(&mut engine, input.arguments) {
                Ok(_) => {}
                Err(error) => println!("{}", error),
            },
            "quit" => break,
            "" => continue,
            _ => println!("Unknown command"),
        }
    }
}

fn uci() {
    println!("id name Pineapple");
    println!("id author Sebastian S.");
    println!("uciok");
}

fn position(engine: &mut Engine, arguments: Vec<&str>) -> Result<(), InputError> {
    if arguments.is_empty() {
        return Err(InputError::InvalidPositionArguments);
    }

    let moves_starting_index = match arguments[0] {
        "fen" => {
            let fen: Vec<&str> = arguments.clone().drain(1..7).collect();
            let fen = fen.join(" ");

            engine.load_position(fen.as_str())?;

            7
        }
        "startpos" => {
            engine.load_position(arguments[0])?;

            1
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
        make_move(engine, move_string)?;
    }

    Ok(())
}

fn make_move(engine: &mut Engine, move_string: &str) -> Result<(), InputError> {
    let mv = engine.find_move_from_string(move_string)?;

    engine.make_move(&mv)?;

    Ok(())
}

#[derive(Debug)]
pub enum InputError {
    IllegalMove(String),
    InvalidFen(FenError),
    InvalidMoveFlag,
    InvalidMoveString(String),
    InvalidPositionArguments,
    MoveNotFound(String),
}

impl Display for InputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputError::IllegalMove(move_string) => {
                write!(f, "Attempted to play illegal move {}", move_string)
            }
            InputError::InvalidFen(error) => write!(f, "Failed to parse FEN: {}", error),
            InputError::InvalidMoveFlag => write!(f, "Invalid move flag"),
            InputError::InvalidMoveString(move_string) => {
                write!(f, "Failed to parse move string {}", move_string)
            }
            InputError::InvalidPositionArguments => {
                write!(f, "Invalid position command arguments")
            }
            InputError::MoveNotFound(move_string) => {
                write!(f, "Failed to find move {}", move_string)
            }
        }
    }
}

#[derive(Debug)]
pub enum FenError {
    BoardPosition,
    SideToMove,
    CastlingRights,
    EnPassantSquare,
}

impl Display for FenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FenError::BoardPosition => write!(f, "unable to parse board position"),
            FenError::SideToMove => write!(f, "unable to parse side to move"),
            FenError::CastlingRights => write!(f, "unable to parse castling rights"),
            FenError::EnPassantSquare => write!(f, "unable to parse en passant square"),
        }
    }
}
