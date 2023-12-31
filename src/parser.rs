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
use crate::grammar::Object::*;
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
        while self.token_match(&[TokenType::BangEqual, TokenType::EqualEqual]) {
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
        while self.token_match(&[
            TokenType::Less,
            TokenType::LessEqual,
            TokenType::Greater,
            TokenType::GreaterEqual,
        ]) {
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
        while self.token_match(&[TokenType::Minus, TokenType::Plus]) {
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
        while self.token_match(&[TokenType::Slash, TokenType::Star]) {
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
        if self.token_match(&[TokenType::Minus, TokenType::Bang]) {
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
        match self.peek().r#type {
            TokenType::True
            | TokenType::False
            | TokenType::String(s)
            | TokenType::Nil
            | TokenType::Number(x) => {
                self.advance();
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
            _ => panic!("unexpected token while parsing"),
        }
    }

    fn consume(&mut self, token_type: TokenType, message: String) -> ParseResult {
        if self.check(&token_type) {
            Ok(self.advance().unwrap())
        } else {
            Err(ParseError::new(self.peek(), message))
        }
    }

    #[allow(dead_code)]
    fn synchronize(&mut self) {
        // consuming the problematic token
        // if it was a semicolon we can synchronize directly there
        self.advance();

        while !self.is_at_end() {
            if self.previous().unwrap().r#type == TokenType::Semicolon {
                return;
            }
            match self.peek().r#type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn token_match(&mut self, types: &[TokenType]) -> bool {
        for token_type in types.iter() {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&mut self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        &self.peek().r#type == token_type
    }

    fn is_at_end(&self) -> bool {
        self.peek().r#type == TokenType::Eof
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn advance(&mut self) -> Option<Token> {
        if !self.is_at_end() {
            self.current += 1;
            self.previous()
        } else {
            None
        }
    }

    fn previous(&mut self) -> Option<Token> {
        self.tokens.get(self.current - 1).cloned()
    }
}
