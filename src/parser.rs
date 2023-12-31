// // expression     → equality ;
// // equality       → comparison ( ( "!=" | "==" ) comparison )* ;
// // comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// // term           → factor ( ( "-" | "+" ) factor )* ;
// // factor         → unary ( ( "/" | "*" ) unary )* ;
// // unary          → ( "!" | "-" ) unary
// //                | primary ;
// // primary        → NUMBER | STRING | "true" | "false" | "nil"
// //                | "(" expression ")" ;
//
// #![allow(dead_code)]
//
// use crate::grammar::{Binary, Expression, Grouping, Literal, Object, Unary};
// use crate::scanner::{Token, TokenType};
// use colored::Colorize;
// use std::fmt::{Display, Formatter};
// use std::process;
//
// struct ParseError {
//     token: Token,
//     message: String,
// }
//
// impl ParseError {
//     fn new(token: Token, message: String) -> Self {
//         Self { token, message }
//     }
// }
//
// impl Display for ParseError {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         let pos = format!("{}:{}:", self.token.line, self.token.col);
//         write!(
//             f,
//             "{} {} ({:?}) {}",
//             pos.bold(),
//             "parsing error:".red(),
//             self.token.r#type,
//             self.message
//         )
//     }
// }
//
// type ParseResult = Result<Token, ParseError>;
//
// pub struct Parser<'a> {
//     tokens: &'a [Token],
//     current: usize,
// }
//
// impl Expression for Box<dyn Expression> {
//     fn print(&self) -> String {
//         self.as_ref().print()
//     }
//
//     fn rpn(&self) -> String {
//         self.as_ref().rpn()
//     }
// }
//
// impl Display for Box<dyn Expression> {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", *self)
//     }
// }
//
// impl<'a> Parser<'a> {
//     pub fn new(tokens: &'a [Token]) -> Self {
//         Self { tokens, current: 0 }
//     }
//
//     pub fn parse(&mut self) -> Box<dyn Expression> {
//         self.expression()
//     }
//
//     fn expression(&mut self) -> Box<dyn Expression> {
//         self.equality()
//     }
//
//     fn equality(&mut self) -> Box<dyn Expression> {
//         let mut expr = self.comparison();
//         while self.token_match(&[TokenType::BangEqual, TokenType::EqualEqual]) {
//             let op = self.previous().unwrap();
//             let right = self.comparison();
//             expr = Box::new(Binary::new(expr, op, right));
//         }
//         expr
//     }
//
//     fn comparison(&mut self) -> Box<dyn Expression> {
//         let mut expr = self.term();
//         while self.token_match(&[
//             TokenType::Less,
//             TokenType::LessEqual,
//             TokenType::Greater,
//             TokenType::GreaterEqual,
//         ]) {
//             let op = self.previous().unwrap();
//             let right = self.term();
//             expr = Box::new(Binary::new(expr, op, right));
//         }
//         expr
//     }
//
//     fn term(&mut self) -> Box<dyn Expression> {
//         let mut expr = self.factor();
//         while self.token_match(&[TokenType::Minus, TokenType::Plus]) {
//             let op = self.previous().unwrap();
//             let right = self.factor();
//             expr = Box::new(Binary::new(expr, op, right));
//         }
//         expr
//     }
//
//     fn factor(&mut self) -> Box<dyn Expression> {
//         let mut expr = self.unary();
//         while self.token_match(&[TokenType::Slash, TokenType::Star]) {
//             let op = self.previous().unwrap();
//             let right = self.unary();
//             expr = Box::new(Binary::new(expr, op, right));
//         }
//         expr
//     }
//
//     fn unary(&mut self) -> Box<dyn Expression> {
//         if self.token_match(&[TokenType::Minus, TokenType::Bang]) {
//             let op = self.previous().unwrap();
//             let right = self.unary();
//             Box::new(Unary::new(op, right))
//         } else {
//             Box::new(self.primary())
//         }
//     }
//
//     fn primary(&mut self) -> Box<dyn Expression> {
//         match self.peek().r#type {
//             TokenType::False => Box::new(Literal::new(Object::Bool(false))),
//             TokenType::True => Box::new(Literal::new(Object::Bool(true))),
//             TokenType::String(s) => Box::new(Literal::new(Object::String(s))),
//             TokenType::Nil => Box::new(Literal::new(Object::Nil)),
//             TokenType::Number(x) => Box::new(Literal::new(Object::Number(x))),
//             TokenType::LeftParen => {
//                 let expr = self.expression();
//                 if let Err(e) = self.consume(
//                     TokenType::RightParen,
//                     "missing `)` after expression".to_string(),
//                 ) {
//                     eprintln!("error: {e}");
//                     process::exit(1);
//                 }
//                 Box::new(Grouping::new(expr))
//             }
//             _ => panic!("unexpected token while parsing"),
//         }
//     }
//
//     fn consume(&mut self, token_type: TokenType, message: String) -> ParseResult {
//         if self.check(&token_type) {
//             Ok(self.advance().unwrap())
//         } else {
//             Err(ParseError::new(self.peek(), message))
//         }
//     }
//
//     fn synchronize(&mut self) {
//         // consuming the problematic token
//         // if it was a semicolon we can synchronize directly there
//         self.advance();
//
//         while !self.is_at_end() {
//             if self.previous().unwrap().r#type == TokenType::Semicolon {
//                 return;
//             }
//             match self.peek().r#type {
//                 TokenType::Class
//                 | TokenType::Fun
//                 | TokenType::Var
//                 | TokenType::For
//                 | TokenType::If
//                 | TokenType::While
//                 | TokenType::Print
//                 | TokenType::Return => return,
//                 _ => {
//                     self.advance();
//                 }
//             }
//         }
//     }
//
//     fn token_match(&mut self, types: &[TokenType]) -> bool {
//         for token_type in types.iter() {
//             if self.check(token_type) {
//                 self.advance();
//                 return true;
//             }
//         }
//         false
//     }
//
//     fn check(&self, token_type: &TokenType) -> bool {
//         if self.is_at_end() {
//             return false;
//         }
//         &self.peek().r#type == token_type
//     }
//
//     fn is_at_end(&self) -> bool {
//         self.peek().r#type == TokenType::Eof
//     }
//
//     fn peek(&self) -> Token {
//         self.tokens[self.current].clone()
//     }
//
//     fn advance(&mut self) -> Option<Token> {
//         if !self.is_at_end() {
//             self.current += 1;
//             dbg!(self.previous())
//         } else {
//             None
//         }
//     }
//
//     fn previous(&mut self) -> Option<Token> {
//         self.tokens.get(self.current - 1).cloned()
//     }
// }
