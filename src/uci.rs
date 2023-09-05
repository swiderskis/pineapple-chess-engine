use crate::engine;
use std::io;

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
        let input = Input::new(input);

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

fn uci() {
    println!("id name Pineapple");
    println!("id author Sebastian S.");
    println!("uciok");
}
