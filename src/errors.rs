use crate::scanner::Token;
use colored::Colorize;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[allow(dead_code)]
#[derive(Debug)]
pub enum LoxError {
    Scan(ScanError),
    Parse(ParseError),
    Runtime(RuntimeError),
}

use LoxError::*;
impl Error for LoxError {}
pub type LoxResult<T> = Result<T, LoxError>;

impl LoxError {
    fn why(&self) -> String {
        match self {
            Scan(ScanError {
                line,
                col,
                message,
                r#type,
            }) => {
                format!("{}:{}: {} ({:?}", line, col, message, r#type)
            }
            Parse(ParseError { token, message }) => format!("{} (on token {:?}", message, token),
            Runtime(RuntimeError { message }) => message.to_owned(),
        }
    }
}

impl From<RuntimeError> for LoxError {
    fn from(value: RuntimeError) -> Self {
        Runtime(value)
    }
}

impl From<ParseError> for LoxError {
    fn from(value: ParseError) -> Self {
        Parse(value)
    }
}

impl From<ScanError> for LoxError {
    fn from(value: ScanError) -> Self {
        Scan(value)
    }
}

impl Display for LoxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let prefix = match self {
            Scan(_) | Parse(_) => "syntax error:",
            Runtime(_) => "runtime error:",
        };
        write!(f, "{} {}", prefix.red(), self.why())
    }
}

// ----------------

#[derive(Debug)]
pub struct RuntimeError {
    message: String,
}

impl RuntimeError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", "runtime error:".red(), self.message)
    }
}

#[derive(Debug)]
pub struct ParseError {
    token: Token,
    message: String,
}

impl ParseError {
    pub fn new(token: Token, message: String) -> Self {
        Self { token, message }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let pos = format!("{}:{}:", self.token.line, self.token.col);
        write!(
            f,
            "{} {} {} (on token `{:?}`)",
            pos.bold(),
            "parsing error:".red(),
            self.message,
            self.token.r#type,
        )
    }
}

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, PartialEq)]
pub enum ScanErrorType {
    UnexpectedCharacter,
    InvalidNumber,
    UnterminatedString,
}

impl Display for ScanErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanErrorType::UnexpectedCharacter => write!(f, "unexpected character"),
            ScanErrorType::InvalidNumber => write!(f, "invalid number"),
            ScanErrorType::UnterminatedString => write!(f, "unterminated string"),
        }
    }
}

#[derive(Debug)]
pub struct ScanError {
    pub line: usize,
    pub col: usize,
    pub message: String,
    pub r#type: ScanErrorType,
}

impl ScanError {
    pub fn new(line: usize, col: usize, message: String, r#type: ScanErrorType) -> Self {
        Self {
            line,
            col,
            message,
            r#type,
        }
    }
}

impl Display for ScanError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let pos = format!("{}:{}:", self.line, self.col);
        write!(
            f,
            "{} {} ({}) {}",
            pos.bold(),
            "syntax error:".red(),
            self.r#type,
            self.message
        )
    }
}

pub type ScanResult<T> = Result<T, ScanError>;
