use std::env;
use std::process;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    match args.len() {
        1 => rlox::run_prompt(),
        2 => rlox::run_file(&args[1]),
        _ => {
            eprintln!("Usage: rlox [script]");
            process::exit(64);
        }
    }
}
