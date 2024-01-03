use std::process;
use std::{env, fs};

fn main() {
    let args = env::args().collect::<Vec<String>>();
    match args.len() {
        1 => rlox::run_prompt(),
        2 => {
            let filename = &args[1];
            let source = match fs::read_to_string(filename) {
                Err(why) => {
                    eprintln!("cannot open {filename}: {why}");
                    process::exit(1);
                }
                Ok(source) => source,
            };
            rlox::run_source(&source);
        }
        3 => {
            let option = &args[1];
            if option != "-c" {
                eprintln!("invalid argument: {option}");
                process::exit(1);
            }
            rlox::run_source(&args[2]);
        }
        _ => {
            eprintln!("Usage: rlox [<filename> | -c <source>]");
            process::exit(64);
        }
    }
}
