pub mod errors;
pub mod expression;
pub mod functions;
pub mod interpreter;
pub mod parser;
pub mod scanner;

use crate::errors::LoxResult;
use crate::interpreter::{Environment, Interpreter};
use crate::parser::Parser;
use crate::scanner::Scanner;
use std::io::Write;
use std::{io, process};

pub fn run_source(source: &str) {
    let mut env = Environment::new();
    if let Err(e) = run(source.trim(), &mut env) {
        eprintln!("{e}");
        process::exit(65);
    }
}

pub fn run_prompt() {
    let mut env = Environment::new();
    loop {
        print!("> ");
        io::stdout().flush().expect("could not flush output stream");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("could not read line");
        if let Err(e) = run(input.trim(), &mut env) {
            eprintln!("{e}");
        }
        input.clear();
    }
}

fn run(source: &str, env: &mut Environment) -> LoxResult<()> {
    let mut scanner = Scanner::new(source);
    scanner.scan_tokens()?;
    let mut parser = Parser::new(scanner.tokens);
    let result = parser.parse();
    match result {
        Ok(statements) => {
            let interpreter = Interpreter::new();
            if let Err(why) = interpreter.interpret(env, &statements) {
                eprintln!("{why}");
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("{e}");
        }
    }
    Ok(())
}
