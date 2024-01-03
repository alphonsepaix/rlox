pub mod grammar;
pub mod interpreter;
pub mod parser;
pub mod scanner;

use crate::interpreter::{Environment, Interpreter};
use crate::parser::Parser;
use crate::scanner::{ScanResult, Scanner};
use std::io::Write;
use std::{fs, io, process};

pub enum Context {
    Repl,
    File,
}

pub fn run_file(filename: &str) {
    let mut env = Environment::new();
    let source = &fs::read_to_string(filename).expect("could not read file");
    if let Err(e) = run(source, &mut env) {
        eprintln!("{e}");
        process::exit(65);
    }
}

pub fn run_prompt() {
    let mut input = String::new();
    let mut env = Environment::new();
    loop {
        print!("> ");
        io::stdout().flush().expect("could not flush output stream");
        io::stdin()
            .read_line(&mut input)
            .expect("could not read line");
        if let Err(e) = run(input.trim(), &mut env) {
            eprintln!("{e}");
        }
        input.clear();
    }
}

fn run(source: &str, env: &mut Environment) -> ScanResult<()> {
    let mut scanner = Scanner::new(source);
    scanner.scan_tokens()?;
    let mut parser = Parser::new(scanner.tokens());
    let result = parser.parse();
    match result {
        Ok(statements) => {
            let interpreter = Interpreter::new();
            interpreter.interpret(env, &statements);
        }
        Err(e) => {
            eprintln!("{e}");
        }
    }
    Ok(())
}
