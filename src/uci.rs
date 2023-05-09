//! # UCI
//!
//! A library to handle inputs following the [UCI protocol](https://backscattering.de/chess/uci/).

use crate::engine;
use std::io;

/// Holds the user input
struct Input<'a> {
    /// The command entered by the user
    command: &'a str,
    /// Any arguments passed by the user
    arguments: Vec<&'a str>,
}

/// Takes a user input, and performs an action based on the command entered
pub fn command() {
    loop {
        let mut input = String::new();

        match io::stdin().read_line(&mut input) {
            Ok(_) => {}
            Err(err) => {
                println!("Error parsing command: {err}");
                continue;
            }
        };

        let input: Vec<&str> = input.split_whitespace().collect();
        let input = Input {
            command: input[0],
            arguments: input[1..].to_vec(),
        };

        match input.command.trim() {
            "uci" => uci(),
            "isready" => println!("readyok"),
            "ucinewgame" => continue,
            "position" => engine::position(),
            "quit" => break,
            "" => continue,
            _ => println!("Unknown command"),
        }
    }
}

/// Prints engine information and returns `uciok` to connected GUI
fn uci() {
    println!("id name Pineapple");
    println!("id author Sebastian S.");
    println!("uciok");
}
