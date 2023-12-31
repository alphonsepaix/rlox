pub mod grammar;
pub mod parser;
pub mod scanner;

use crate::grammar::Representation;
use crate::parser::Parser;
use crate::scanner::{ScanResult, Scanner};
use std::io::Write;
use std::{fs, io, process};

pub fn run_file(filename: &str) {
    let source = &fs::read_to_string(filename).expect("could not read file");
    if let Err(e) = run(source) {
        eprintln!("{e}");
        process::exit(65);
    }
}

pub fn run_prompt() {
    let mut input = String::new();
    loop {
        print!("> ");
        io::stdout().flush().expect("could not flush output stream");
        io::stdin()
            .read_line(&mut input)
            .expect("could not read line");
        if let Err(e) = run(input.trim()) {
            eprintln!("{e}");
        }
        input.clear();
    }
}

fn run(source: &str) -> ScanResult<()> {
    let mut scanner = Scanner::new(source);
    scanner.scan_tokens()?;
    for token in scanner.tokens() {
        println!("{:?}", token);
    }
    let mut parser = Parser::new(scanner.tokens());
    let result = parser.parse();
    match result {
        Ok(expr) => {
            println!("{}", expr.repr());
        }
        Err(e) => {
            eprintln!("{e}");
        }
    }
    Ok(())
}
