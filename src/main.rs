mod lexer;
mod interpreter;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: tumul <filename.tu>");
        std::process::exit(1);
    }
    let filename = &args[1];
    let code = fs::read_to_string(filename)
        .expect("Failed to read file");
    unimplemented!()
}
