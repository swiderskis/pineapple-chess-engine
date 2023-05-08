fn main() {
    uci::command();
}

mod uci {
    use std::io;

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

            let mut input: Vec<&str> = input.split_whitespace().collect();
            let command = input[0];
            let arguments = input.split_off(1);

            match command.trim() {
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
