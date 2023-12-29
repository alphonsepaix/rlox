// #![allow(dead_code)]

use phf::phf_map;
use std::env;
use std::fmt::{Display, Formatter};
use std::fs;
use std::io;
use std::io::Write;
use std::iter::{Enumerate, Peekable};
use std::process;
use std::str::Chars;

static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "and" => TokenType::And,
    "class" => TokenType::Class,
    "else" => TokenType::Else,
    "false" => TokenType::False,
    "for" => TokenType::For,
    "fun" => TokenType::Fn,
    "if" => TokenType::If,
    "nil" => TokenType::Nil,
    "or" => TokenType::Or,
    "print" => TokenType::Print,
    "return" => TokenType::Return,
    "super" => TokenType::Super,
    "this" => TokenType::This,
    "true" => TokenType::True,
    "var" => TokenType::Var,
    "while" => TokenType::While,
};

struct ErrorStruct {
    line: usize,
    col: usize,
    message: String,
}

type LoxResult<T> = Result<T, ErrorStruct>;

impl Display for ErrorStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "error: {} ({}:{})", self.message, self.line, self.col)
    }
}

#[derive(Clone, Debug)]
enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    Identifier(String),
    String(String),
    Number(f64),

    And,
    Class,
    Else,
    Fn,
    For,
    False,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

struct Token {
    r#type: TokenType,
    lexeme: String,
    line: usize,
    col: usize,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let pos = format!("{}:{}", self.line, self.col);
        write!(f, "{:<10} {:?} {}", pos, self.r#type, self.lexeme)
    }
}

struct Scanner<'a> {
    source: &'a str,
    stream: Peekable<Enumerate<Chars<'a>>>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
    fn scan_tokens(&mut self) -> LoxResult<()> {
        while !self.at_end() {
            self.start = self.current;
            self.scan_token()?;
        }
        self.tokens.push(Token {
            r#type: TokenType::Eof,
            lexeme: "".to_string(),
            line: self.line,
            col: self.current,
        });
        Ok(())
    }

    fn scan_token(&mut self) -> LoxResult<()> {
        let (i, c) = self.stream.next().expect("scanning in empty stream");
        self.current = i + 1;
        let r#type = match c {
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            '{' => TokenType::LeftBrace,
            '}' => TokenType::RightBrace,
            ',' => TokenType::Comma,
            '.' => TokenType::Dot,
            '-' => TokenType::Minus,
            '+' => TokenType::Plus,
            ';' => TokenType::Semicolon,
            '*' => TokenType::Star,
            '!' => {
                if self.next_match('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                }
            }
            '=' => {
                if self.next_match('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                }
            }
            '>' => {
                if self.next_match('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                }
            }
            '<' => {
                if self.next_match('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                }
            }
            '/' => {
                if self.next_match('/') {
                    while !self.at_end() && self.peek() != '\n' {
                        self.stream.next().unwrap();
                    }
                    return Ok(());
                }
                TokenType::Slash
            }
            ' ' | '\r' | '\t' => return Ok(()),
            '\n' => {
                self.line += 1;
                return Ok(());
            }
            '"' => self.string()?,
            x if x.is_ascii_digit() => self.number()?,
            c if c.is_ascii_alphabetic() => self.identifier()?,
            _ => {
                return Err(ErrorStruct {
                    line: self.line,
                    col: self.current,
                    message: "unexpected character".to_string(),
                })
            }
        };
        self.add_token(r#type);
        Ok(())
    }

    fn add_token(&mut self, r#type: TokenType) {
        self.tokens.push(Token {
            r#type,
            lexeme: self.source[self.start..self.current].to_owned(),
            line: self.line,
            col: self.current,
        });
    }

    fn at_end(&mut self) -> bool {
        return self.stream.peek() == None;
    }

    fn peek(&mut self) -> char {
        self.stream.peek().expect("stream should not be empty").1
    }

    fn next_match(&mut self, expected: char) -> bool {
        if self.at_end() {
            return false;
        }

        if self.peek() != expected {
            false
        } else {
            self.current += 1;
            self.stream.next().unwrap();
            true
        }
    }

    fn string(&mut self) -> LoxResult<TokenType> {
        let mut s = String::new();
        self.start += 1;
        while !self.at_end() && self.peek() != '"' {
            let (i, c) = self.stream.next().unwrap();
            self.current = i + 1;
            s.push(c);
            if c == '\n' {
                self.line += 1;
            }
        }
        if self.at_end() {
            return Err(ErrorStruct {
                line: self.line,
                col: self.current,
                message: "unterminated string".to_string(),
            });
        }
        self.stream.next().unwrap();
        Ok(TokenType::String(s))
    }

    fn number(&mut self) -> LoxResult<TokenType> {
        while !self.at_end() && self.peek().is_ascii_digit() {
            self.current = self.stream.next().unwrap().0 + 1;
        }
        if !self.at_end() && self.peek() == '.' {
            self.stream.next().unwrap();
            if self.at_end() || !self.peek().is_ascii_digit() {
                return Err(ErrorStruct {
                    line: self.line,
                    col: self.current,
                    message: "invalid number".to_string(),
                });
            }
            while !self.at_end() && self.peek().is_ascii_digit() {
                self.current = self.stream.next().unwrap().0 + 1;
            }
        }
        let num = self.source[self.start..self.current]
            .parse::<f64>()
            .unwrap();
        Ok(TokenType::Number(num))
    }

    fn identifier(&mut self) -> LoxResult<TokenType> {
        while !self.at_end() && self.peek().is_ascii_alphabetic() {
            self.current = self.stream.next().unwrap().0 + 1;
        }
        let s = &self.source[self.start..self.current];
        let r#type = KEYWORDS
            .get(s)
            .cloned()
            .unwrap_or(TokenType::Identifier(s.to_owned()));
        Ok(r#type)
    }

    fn new(source: &'a str) -> Self {
        let stream = source.chars().enumerate().peekable();
        Scanner {
            source,
            stream,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
        }
    }
}

fn main() {
    let args = env::args().collect::<Vec<String>>();
    if args.len() > 2 {
        eprintln!("usage: rlox [script]");
        process::exit(64);
    } else if args.len() == 2 {
        let filename = &args[1];
        run_file(filename);
    } else {
        run_prompt();
    }
}

fn run_file(filename: &str) {
    let source = &fs::read_to_string(filename).expect("could not read file");
    if let Err(e) = run(source) {
        eprintln!("{e}");
        process::exit(65);
    }
}

fn run_prompt() {
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

fn run(source: &str) -> LoxResult<()> {
    let mut scanner = Scanner::new(source);
    scanner.scan_tokens()?;
    for token in scanner.tokens {
        println!("{}", token);
    }
    Ok(())
}
