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
                format!("{}:{}: {} ({})", line, col, message, r#type)
            }
            Parse(ParseError { token, message }) => format!("{} (on token `{}`)", message, token),
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
    pub fn build(message: String) -> LoxError {
        Runtime(Self { message })
    }
}

#[derive(Debug)]
pub struct ParseError {
    token: Token,
    message: String,
}

impl ParseError {
    pub fn build(token: Token, message: String) -> LoxError {
        Parse(Self { token, message })
    }
}

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
    pub fn build(line: usize, col: usize, message: String, r#type: ScanErrorType) -> LoxError {
        Scan(Self {
            line,
            col,
            message,
            r#type,
        })
    }
}
