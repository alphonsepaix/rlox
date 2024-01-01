// program        → declaration* EOF ;

// declaration    → varDecl
//                | statement ;
// varDecl        → "var" IDENTIFIER ( "=" expression )? ";" ;
//
// statement      → exprStmt
//                | printStmt ;
// exprStmt       → expression ";" ;
// printStmt      → "print" expression ";" ;
//
// expression     → equality ;
// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term           → factor ( ( "-" | "+" ) factor )* ;
// factor         → unary ( ( "/" | "*" ) unary )* ;
// unary          → ( "!" | "-" ) unary
//                | primary ;
// primary        → NUMBER | STRING | "true" | "false" | "nil"
//                | "(" expression ")"
//                | IDENTIFIER ;

use crate::grammar::Expression;
use crate::grammar::Expression::*;
use crate::grammar::Stmt;
use crate::scanner::{Token, TokenType};
use colored::Colorize;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct ParseError {
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
            "{} {} {} (on token `{:?}`)",
            pos.bold(),
            "parsing error:".red(),
            self.message,
            self.token.r#type,
        )
    }
}

impl Error for ParseError {}

type ParseResult<T> = Result<T, ParseError>;

pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut statements = vec![];
        while self.peek_type() != TokenType::Eof {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> ParseResult<Stmt> {
        let statement = if self.peek_type() == TokenType::Var {
            self.advance();
            self.var_declaration()
        } else {
            self.statement()
        };
        statement.map_err(|e| {
            self.synchronize();
            e
        })
    }

    fn var_declaration(&mut self) -> ParseResult<Stmt> {
        if let TokenType::Identifier(name) = self.peek_type() {
            self.advance();
            let mut initializer: Option<Expression> = None;
            if self.peek_type() == TokenType::Equal {
                self.advance();
                initializer = Some(self.expression()?);
            }
            self.consume(
                TokenType::Semicolon,
                "expected `;` after variable declaration".to_string(),
            )?;
            Ok(Stmt::Var { name, initializer })
        } else {
            Err(ParseError::new(
                self.peek(),
                "expected variable name".to_string(),
            ))
        }
    }

    fn statement(&mut self) -> ParseResult<Stmt> {
        if self.peek_type() == TokenType::Print {
            self.advance();
            self.print_statement()
        } else {
            self.expr_statement()
        }
    }

    fn print_statement(&mut self) -> ParseResult<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "expected `;` after value".to_string())?;
        Ok(Stmt::Print(expr))
    }

    fn expr_statement(&mut self) -> ParseResult<Stmt> {
        let expr = self.expression()?;
        self.consume(
            TokenType::Semicolon,
            "expected `;` after expression".to_string(),
        )?;
        Ok(Stmt::Expr(expr))
    }

    fn expression(&mut self) -> ParseResult<Expression> {
        self.equality()
    }

    fn equality(&mut self) -> ParseResult<Expression> {
        let mut expr = self.comparison()?;
        while matches!(
            self.peek_type(),
            TokenType::BangEqual | TokenType::EqualEqual
        ) {
            self.advance();
            let op = self.previous().unwrap();
            let right = self.comparison()?;
            expr = Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> ParseResult<Expression> {
        let mut expr = self.term()?;
        while matches!(
            self.peek_type(),
            TokenType::Less | TokenType::LessEqual | TokenType::Greater | TokenType::GreaterEqual
        ) {
            self.advance();
            let op = self.previous().unwrap();
            let right = self.term()?;
            expr = Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn term(&mut self) -> ParseResult<Expression> {
        let mut expr = self.factor()?;
        while matches!(self.peek_type(), TokenType::Minus | TokenType::Plus) {
            self.advance();
            let op = self.previous().unwrap();
            let right = self.factor()?;
            expr = Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn factor(&mut self) -> ParseResult<Expression> {
        let mut expr = self.unary()?;
        while matches!(self.peek_type(), TokenType::Slash | TokenType::Star) {
            self.advance();
            let op = self.previous().unwrap();
            let right = self.unary()?;
            expr = Binary {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn unary(&mut self) -> ParseResult<Expression> {
        if matches!(self.peek_type(), TokenType::Minus | TokenType::Bang) {
            self.advance();
            let op = self.previous().unwrap();
            let right = self.unary()?;
            Ok(Unary {
                op,
                right: Box::new(right),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> ParseResult<Expression> {
        let token_type = self.peek_type();
        match token_type {
            TokenType::True
            | TokenType::False
            | TokenType::Nil
            | TokenType::Number(_)
            | TokenType::String(_) => {
                self.advance();
                Ok(token_type.try_into().unwrap())
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(
                    TokenType::RightParen,
                    "missing `)` after expression".to_string(),
                )?;
                Ok(Grouping(Box::new(expr)))
            }
            TokenType::Identifier(name) => {
                self.advance();
                Ok(Variable(name))
            }
            _ => Err(ParseError::new(
                self.peek(),
                "unexpected token while parsing".to_string(),
            )),
        }
    }

    fn consume(&mut self, token_type: TokenType, message: String) -> ParseResult<Token> {
        if self.peek_type() == token_type {
            self.advance();
            Ok(self.peek())
        } else {
            Err(ParseError::new(self.peek(), message))
        }
    }

    fn synchronize(&mut self) {
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
