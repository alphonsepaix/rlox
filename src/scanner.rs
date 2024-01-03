use crate::errors::{ScanError, ScanErrorType, ScanResult};
use crate::grammar::Expression;
use crate::grammar::Expression::Literal;
use crate::grammar::Object::{Bool, Nil, Number, Str};
use phf::phf_map;
use std::fmt::{Debug, Display, Formatter};
use std::iter::Peekable;
use std::str::Chars;

pub static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "and" => TokenType::And,
    "class" => TokenType::Class,
    "else" => TokenType::Else,
    "false" => TokenType::False,
    "for" => TokenType::For,
    "fun" => TokenType::Fun,
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
    "break" => TokenType::Break,
    "continue" => TokenType::Continue,
};

#[derive(Clone, Debug, PartialEq)]
pub enum TokenType {
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
    Break,
    Class,
    Continue,
    Else,
    Fun,
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

impl TryInto<Expression> for TokenType {
    type Error = ();

    fn try_into(self) -> Result<Expression, Self::Error> {
        match self {
            TokenType::String(s) => Ok(Literal(Str(s))),
            TokenType::Number(x) => Ok(Literal(Number(x))),
            TokenType::False => Ok(Literal(Bool(false))),
            TokenType::Nil => Ok(Literal(Nil)),
            TokenType::True => Ok(Literal(Bool(true))),
            _ => Err(()),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Token {
    pub r#type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub col: usize,
}

impl Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let pos = format!("{}:{}:", self.line, self.col);
        write!(f, "{:<10} {:?} {}", pos, self.r#type, self.lexeme)
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lexeme)
    }
}

pub struct Scanner<'a> {
    source: &'a str,
    stream: Peekable<Chars<'a>>,
    pub tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    col: usize,
}

impl<'a> Scanner<'a> {
    pub fn scan_tokens(&mut self) -> ScanResult<()> {
        while self.peek().is_some() {
            self.start = self.current;
            self.scan_token()?;
        }
        self.add_eof_token();
        Ok(())
    }

    fn scan_token(&mut self) -> ScanResult<()> {
        let c = self.advance().expect("scanning in empty stream");
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
                    while let Some(c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.advance();
                    }
                    return Ok(());
                }
                TokenType::Slash
            }
            ' ' | '\r' | '\t' | '\n' => {
                return Ok(());
            }
            '"' => self.string()?,
            x if x.is_ascii_digit() => self.number()?,
            c if c.is_ascii_alphabetic() => self.identifier()?,
            _ => {
                return Err(self.error(
                    ScanErrorType::UnexpectedCharacter,
                    "unexpected symbol while parsing",
                ))
            }
        };
        self.add_token(r#type);
        Ok(())
    }

    fn add_token(&mut self, r#type: TokenType) {
        let lexeme = self.source[self.start..self.current].to_owned();
        let start = self.col - lexeme.len();
        self.tokens.push(Token {
            r#type,
            lexeme,
            line: self.line,
            col: start,
        });
    }

    fn add_eof_token(&mut self) {
        self.tokens.push(Token {
            r#type: TokenType::Eof,
            lexeme: "".to_string(),
            line: self.line,
            col: self.col,
        });
    }

    fn peek(&mut self) -> Option<char> {
        self.stream.peek().cloned()
    }

    fn advance(&mut self) -> Option<char> {
        self.current += 1;
        self.col += 1;
        let next = self.stream.next();
        if let Some(c) = next {
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            }
        }
        next
    }

    fn next_match(&mut self, expected: char) -> bool {
        match self.peek() {
            None => false,
            Some(c) => {
                if c != expected {
                    false
                } else {
                    self.advance();
                    true
                }
            }
        }
    }

    fn string(&mut self) -> ScanResult<TokenType> {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c == '"' {
                break;
            }
            let c = self.advance().unwrap();
            s.push(c);
        }
        if self.peek().is_none() {
            return Err(self.error(ScanErrorType::UnterminatedString, "missing \" delimiter"));
        }
        self.advance();
        Ok(TokenType::String(s))
    }

    fn number(&mut self) -> ScanResult<TokenType> {
        while let Some(c) = self.peek() {
            if !c.is_ascii_digit() {
                break;
            }
            self.advance();
        }
        if let Some('.') = self.peek() {
            self.advance();
            // if the stream is empty we put a random alpha character to make sure that the next test fails
            let next = self.advance().unwrap_or('a');
            if !next.is_ascii_digit() {
                return Err(self.error(ScanErrorType::InvalidNumber, "invalid decimal part"));
            } else {
                while let Some(c) = self.peek() {
                    if !c.is_ascii_digit() {
                        break;
                    }
                    self.advance();
                }
            }
        }
        let num = self.source[self.start..self.current]
            .parse::<f64>()
            .unwrap();
        Ok(TokenType::Number(num))
    }

    fn identifier(&mut self) -> ScanResult<TokenType> {
        while let Some(c) = self.peek() {
            if !c.is_ascii_alphabetic() {
                break;
            }
            self.advance();
        }
        let s = &self.source[self.start..self.current];
        let r#type = KEYWORDS
            .get(s)
            .cloned()
            .unwrap_or(TokenType::Identifier(s.to_owned()));
        Ok(r#type)
    }

    pub fn new(source: &'a str) -> Self {
        let stream = source.chars().peekable();
        Scanner {
            source,
            stream,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
            col: 1,
        }
    }

    fn error(&self, r#type: ScanErrorType, message: &str) -> ScanError {
        ScanError {
            line: self.line,
            col: self.col,
            message: message.to_string(),
            r#type,
        }
    }
}
