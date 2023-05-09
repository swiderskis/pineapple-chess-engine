fn main() {
    uci::command();
}

mod uci {
    use std::io;

    struct Input<'a> {
        command: &'a str,
        arguments: Vec<&'a str>,
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
            let input = Input {
                command: input[0],
                arguments: input[1..].to_vec(),
            };

            match input.command.trim() {
                "uci" => uci(),
                "isready" => println!("readyok"),
                "ucinewgame" => continue,
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
}
