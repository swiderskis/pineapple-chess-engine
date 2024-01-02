use crate::engine::Engine;
use std::{fmt::Display, io, str::FromStr, sync::mpsc, thread, time::Duration};

const STARTPOS_MOVES_STARTING_INDEX: usize = 1;
const FEN_MOVES_STARTING_INDEX: usize = 7;

const DEFAULT_DEPTH: u8 = 64;
const DEFAULT_MOVES_TO_GO: u64 = 30;

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
    let (stop_search_sender, stop_search_receiver) = mpsc::channel();
    let (input_sender, input_receiver) = mpsc::channel();
    engine.set_stop_search_receiver(stop_search_receiver);

    thread::spawn(move || loop {
        let mut input = String::new();

        match io::stdin().read_line(&mut input) {
            Ok(_) => match input.trim() {
                "stop" => _ = stop_search_sender.send(true),
                _ => _ = input_sender.send(Some(input)),
            },
            Err(_) => _ = input_sender.send(None),
        }
    });

    loop {
        let input = match input_receiver.recv() {
            Ok(input) => match input {
                Some(input) => input,
                None => continue,
            },
            Err(_) => continue,
        };
        let input = Input::new(&input);

        match input.command {
            "uci" => uci(),
            "isready" => println!("readyok"),
            "ucinewgame" => engine.reset_game(),
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

fn position(engine: &mut Engine, arguments: Vec<&str>) -> Result<(), InputError> {
    if arguments.is_empty() {
        return Err(InputError::InvalidPositionArguments);
    }

    let moves_starting_index = match arguments[0] {
        "startpos" => {
            engine.load_fen(&arguments)?;

            STARTPOS_MOVES_STARTING_INDEX
        }
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
    let mut depth = DEFAULT_DEPTH;
    let mut increment = None;
    let mut move_time = None;
    let mut time_left = None;
    let mut moves_to_go = DEFAULT_MOVES_TO_GO;

    for (index, argument) in arguments.iter().enumerate() {
        match *argument {
            "depth" => {
                depth = get_argument_value(
                    &arguments,
                    index,
                    InputError::InvalidGoArguments(GoArgumentError::Depth),
                )?;

                if depth == 0 {
                    return Err(InputError::InvalidGoArguments(GoArgumentError::Depth));
                }
            }
            "winc" | "binc" => {
                let increment_ms = get_argument_value(
                    &arguments,
                    index,
                    InputError::InvalidGoArguments(GoArgumentError::Increment(
                        argument.to_string(),
                    )),
                )?;
                increment = Some(Duration::from_millis(increment_ms));
            }
            "movetime" => {
                let move_time_ms = get_argument_value(
                    &arguments,
                    index,
                    InputError::InvalidGoArguments(GoArgumentError::MoveTime),
                )?;
                move_time = Some(Duration::from_millis(move_time_ms));
            }
            "wtime" | "btime" => {
                let time_left_ms = get_argument_value(
                    &arguments,
                    index,
                    InputError::InvalidGoArguments(GoArgumentError::TimeLeft(argument.to_string())),
                )?;
                time_left = Some(Duration::from_millis(time_left_ms));
            }

            "movestogo" => {
                moves_to_go = get_argument_value(
                    &arguments,
                    index,
                    InputError::InvalidGoArguments(GoArgumentError::MovesToGo),
                )?;
            }
            "infinite" => {}
            _ => continue,
        }
    }

    engine.set_search_timing(increment, move_time, time_left, moves_to_go);

    let best_move = engine.search_best_move(depth)?.as_string();
    println!("bestmove {}", best_move);

    Ok(())
}

fn make_move_from_string(engine: &mut Engine, move_string: &str) -> Result<(), InputError> {
    engine.make_move(move_string)?;

    Ok(())
}

fn handle_command<F: Fn(&mut Engine, Vec<&str>) -> Result<(), InputError>>(
    command_fn: F,
    engine: &mut Engine,
    arguments: Vec<&str>,
) {
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
    InvalidGoArguments(GoArgumentError),
    InvalidMoveString,
    InvalidPosition,
    InvalidPositionArguments,
}

impl Display for InputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IllegalMove => write!(f, "Attempted to play an illegal move"),
            Self::InvalidFen(error) => write!(f, "Failed to parse FEN: {}", error),
            Self::InvalidGoArguments(error) => write!(f, "Invalid go command argument: {}", error),
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

#[derive(Debug)]
pub enum GoArgumentError {
    Depth,
    Increment(String),
    MoveTime,
    TimeLeft(String),
    MovesToGo,
}

impl Display for GoArgumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GoArgumentError::Depth => write!(f, "depth"),
            GoArgumentError::Increment(argument) => write!(f, "{}", argument),
            GoArgumentError::MoveTime => write!(f, "movetime"),
            GoArgumentError::TimeLeft(argument) => {
                write!(f, "{}", argument)
            }
            GoArgumentError::MovesToGo => write!(f, "movestogo"),
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
