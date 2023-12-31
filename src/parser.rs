// expression     → equality ;
// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term           → factor ( ( "-" | "+" ) factor )* ;
// factor         → unary ( ( "/" | "*" ) unary )* ;
// unary          → ( "!" | "-" ) unary
//                | primary ;
// primary        → NUMBER | STRING | "true" | "false" | "nil"
//                | "(" expression ")" ;

use crate::grammar::Expression::*;
use crate::grammar::*;
use crate::scanner::{Token, TokenType};
use colored::Colorize;
use std::fmt::{Display, Formatter};
use std::process;

struct ParseError {
    token: Token,
    message: String,
}

impl ParseError {
    fn new(token: Token, message: String) -> Self {
        Self { token, message }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let pos = format!("{}:{}:", self.token.line, self.token.col);
        write!(
            f,
            "{} {} ({:?}) {}",
            pos.bold(),
            "parsing error:".red(),
            self.token.r#type,
            self.message
        )
    }
}

type ParseResult = Result<Token, ParseError>;

pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Expression {
        self.expression()
    }

    fn expression(&mut self) -> Expression {
        self.equality()
    }

    fn equality(&mut self) -> Expression {
        let mut expr = self.comparison();
        while matches!(
            self.peek_type(),
            TokenType::BangEqual | TokenType::EqualEqual
        ) {
            self.advance();
            let op = self.previous().unwrap();
            let right = self.comparison();
            expr = Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        expr
    }

    fn comparison(&mut self) -> Expression {
        let mut expr = self.term();
        while matches!(
            self.peek_type(),
            TokenType::Less | TokenType::LessEqual | TokenType::Greater | TokenType::GreaterEqual
        ) {
            self.advance();
            let op = self.previous().unwrap();
            let right = self.term();
            expr = Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        expr
    }

    fn term(&mut self) -> Expression {
        let mut expr = self.factor();
        while matches!(self.peek_type(), TokenType::Minus | TokenType::Plus) {
            self.advance();
            let op = self.previous().unwrap();
            let right = self.factor();
            expr = Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        expr
    }

    fn factor(&mut self) -> Expression {
        let mut expr = self.unary();
        while matches!(self.peek_type(), TokenType::Slash | TokenType::Star) {
            self.advance();
            let op = self.previous().unwrap();
            let right = self.unary();
            expr = Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        expr
    }

    fn unary(&mut self) -> Expression {
        if matches!(self.peek_type(), TokenType::Minus | TokenType::Bang) {
            self.advance();
            let op = self.previous().unwrap();
            let right = self.unary();
            Unary {
                op,
                right: Box::new(right),
            }
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Expression {
        let token_type = self.peek_type();
        match token_type {
            TokenType::True
            | TokenType::False
            | TokenType::Nil
            | TokenType::Number(_)
            | TokenType::String(_) => {
                self.advance();
                token_type.try_into().unwrap()
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.expression();
                if let Err(e) = self.consume(
                    TokenType::RightParen,
                    "missing `)` after expression".to_string(),
                ) {
                    eprintln!("error: {e}");
                    process::exit(1);
                }
                Grouping(Box::new(expr))
            }
            _ => panic!("unknown token while parsing"),
        }
    }

    fn consume(&mut self, token_type: TokenType, message: String) -> ParseResult {
        if self.peek_type() == token_type {
            self.advance();
            Ok(self.peek())
        } else {
            Err(ParseError::new(self.peek(), message))
        }
    }

    #[allow(dead_code)]
    fn synchronize(&mut self) {
        // consuming the problematic token
        // if it was a semicolon we can synchronize directly there
        self.advance();

        while self.peek_type() != TokenType::Eof {
            if self.previous().unwrap().r#type == TokenType::Semicolon {
                return;
            }
            match self.peek_type() {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => self.advance(),
            }
        }
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn peek_type(&self) -> TokenType {
        self.peek().r#type
    }

    fn advance(&mut self) {
        if self.peek_type() != TokenType::Eof {
            self.current += 1;
        }
    }

    fn previous(&mut self) -> Option<Token> {
        self.tokens.get(self.current - 1).cloned()
    }
}
