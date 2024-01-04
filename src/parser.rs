use crate::errors::{LoxResult, ParseError};
use crate::expression::{Expression, Expression::*, Object};
use crate::scanner::{Token, TokenType};

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Var {
        name: String,
        initializer: Option<Expression>,
    },
    Print(Expression),
    Expr(Expression),
    Block(Vec<Stmt>),
    If {
        condition: Expression,
        then_stmt: Box<Stmt>,
        else_stmt: Option<Box<Stmt>>,
    },
    While {
        condition: Expression,
        body: Box<Stmt>,
        increment: Option<Expression>,
    },
    Break,
    Continue,
    Return(Option<Expression>),
    Function {
        name: String,
        body: Vec<Stmt>,
        parameters: Vec<String>,
    },
    Null,
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    enclosing_loops: usize,
    enclosing_funcs: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            enclosing_loops: 0,
            enclosing_funcs: 0,
        }
    }

    pub fn parse(&mut self) -> LoxResult<Vec<Stmt>> {
        let mut statements = vec![];
        while self.peek_type() != TokenType::Eof {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> LoxResult<Stmt> {
        let statement = match self.peek_type() {
            TokenType::Let => {
                self.advance();
                self.var_declaration()
            }
            TokenType::Fn => {
                self.enclosing_funcs += 1;
                self.advance();
                let res = self.function("function");
                self.enclosing_funcs -= 1;
                res
            }
            _ => self.statement(),
        };
        statement.map_err(|e| {
            self.synchronize();
            e
        })
    }

    fn var_declaration(&mut self) -> LoxResult<Stmt> {
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
            Err(ParseError::build(
                self.peek(),
                "expected variable name".to_string(),
            ))
        }
    }

    fn statement(&mut self) -> LoxResult<Stmt> {
        match self.peek_type() {
            TokenType::Print => {
                self.advance();
                self.print_statement()
            }
            TokenType::Semicolon => {
                self.advance();
                Ok(Stmt::Null)
            }
            TokenType::LeftBrace => {
                self.advance();
                self.block().map(Stmt::Block)
            }
            TokenType::If => {
                self.advance();
                self.if_statement()
            }
            TokenType::While => {
                self.enclosing_loops += 1;
                self.advance();
                let res = self.while_statement();
                self.enclosing_loops += 1;
                res
            }
            TokenType::For => {
                self.enclosing_loops += 1;
                self.advance();
                let res = self.for_statement();
                self.enclosing_loops -= 1;
                res
            }
            TokenType::Break => {
                if self.enclosing_loops == 0 {
                    return Err(ParseError::build(
                        self.peek(),
                        "`break` outside loop".to_string(),
                    ));
                }
                self.advance();
                self.consume(
                    TokenType::Semicolon,
                    "expected `;` after `break`".to_string(),
                )?;
                Ok(Stmt::Break)
            }
            TokenType::Continue => {
                if self.enclosing_loops == 0 {
                    return Err(ParseError::build(
                        self.peek(),
                        "`continue` outside loop".to_string(),
                    ));
                }
                self.advance();
                self.consume(
                    TokenType::Semicolon,
                    "expected `;` after `continue`".to_string(),
                )?;
                Ok(Stmt::Continue)
            }
            TokenType::Return => {
                if self.enclosing_funcs == 0 {
                    return Err(ParseError::build(
                        self.peek(),
                        "`return` outside function".to_string(),
                    ));
                }
                self.advance();
                let expr = if self.peek_type() != TokenType::Semicolon {
                    Some(self.expression()?)
                } else {
                    None
                };
                self.consume(
                    TokenType::Semicolon,
                    "expected `;` after `return`".to_string(),
                )?;
                Ok(Stmt::Return(expr))
            }
            _ => self.expr_statement(),
        }
    }

    fn block(&mut self) -> LoxResult<Vec<Stmt>> {
        let mut statements = vec![];

        while self.peek_type() != TokenType::Eof && self.peek_type() != TokenType::RightBrace {
            statements.push(self.declaration()?);
        }

        self.consume(
            TokenType::RightBrace,
            "expected `}` after block".to_string(),
        )?;
        Ok(statements)
    }

    fn print_statement(&mut self) -> LoxResult<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "expected `;` after value".to_string())?;
        Ok(Stmt::Print(expr))
    }

    fn function(&mut self, kind: &str) -> LoxResult<Stmt> {
        let name = self.consume_identifier(format!("expected {kind} name"))?;
        self.consume(
            TokenType::LeftParen,
            format!("expected `(` after {kind} name"),
        )?;
        let mut parameters = vec![];
        if self.peek_type() != TokenType::RightParen {
            loop {
                let parameter = self.consume_identifier("expected parameter name".to_string())?;
                parameters.push(parameter);
                if parameters.len() >= 255 {
                    return Err(ParseError::build(
                        self.previous().unwrap(),
                        "can't have more than 255 parameters".to_string(),
                    ));
                }
                if let TokenType::Comma = self.peek_type() {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.consume(
            TokenType::RightParen,
            "expected `)` after parameters".to_string(),
        )?;
        self.consume(
            TokenType::LeftBrace,
            format!("expected `{{` before {kind} body"),
        )?;
        let body = self.block()?;
        Ok(Stmt::Function {
            name,
            parameters,
            body,
        })
    }

    fn if_statement(&mut self) -> LoxResult<Stmt> {
        self.consume(TokenType::LeftParen, "expected `(` after `if`".to_string())?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "expected `)` after `if`".to_string())?;
        let then_stmt = Box::new(self.statement()?);
        let mut else_stmt = None;
        if let TokenType::Else = self.peek_type() {
            self.advance();
            else_stmt = Some(Box::new(self.statement()?));
        }
        Ok(Stmt::If {
            condition,
            then_stmt,
            else_stmt,
        })
    }

    fn while_statement(&mut self) -> LoxResult<Stmt> {
        self.consume(
            TokenType::LeftParen,
            "expected `(` after `while`".to_string(),
        )?;
        let condition = self.expression()?;
        self.consume(
            TokenType::RightParen,
            "expected `)` after `while`".to_string(),
        )?;
        let stmt = Box::new(self.statement()?);
        Ok(Stmt::While {
            condition,
            body: stmt,
            increment: None,
        })
    }

    fn for_statement(&mut self) -> LoxResult<Stmt> {
        self.consume(TokenType::LeftParen, "expected `(` after `for`".to_string())?;
        let initializer = match self.peek_type() {
            TokenType::Let => {
                self.advance();
                Some(Box::new(self.var_declaration()?))
            }
            TokenType::Semicolon => {
                self.advance();
                None
            }
            _ => {
                self.advance();
                Some(Box::new(self.expr_statement()?))
            }
        };
        let condition = self
            .null_expression(TokenType::Semicolon, "expected `;` after loop condition")?
            .unwrap_or(Literal(Object::Bool(true)));
        let increment =
            self.null_expression(TokenType::RightParen, "expected `)` after for clauses")?;
        let body = self.statement()?;

        let mut statements = vec![];
        if let Some(init) = initializer {
            statements.push(*init);
        }
        let mut while_body = vec![body];
        if let Some(inc) = increment.clone() {
            while_body.push(Stmt::Expr(inc));
        }
        statements.push(Stmt::While {
            condition,
            body: Box::new(Stmt::Block(while_body)),
            increment,
        });
        Ok(Stmt::Block(statements))
    }

    fn null_expression(
        &mut self,
        delimiter: TokenType,
        message: &str,
    ) -> LoxResult<Option<Expression>> {
        if self.peek_type() == delimiter {
            self.advance();
            Ok(None)
        } else {
            let expr = self.expression()?;
            self.consume(delimiter, message.to_string())?;
            Ok(Some(expr))
        }
    }

    fn expr_statement(&mut self) -> LoxResult<Stmt> {
        let expr = self.expression()?;
        self.consume(
            TokenType::Semicolon,
            "expected `;` after expression".to_string(),
        )?;
        Ok(Stmt::Expr(expr))
    }

    fn expression(&mut self) -> LoxResult<Expression> {
        self.assignment()
    }

    fn assignment(&mut self) -> LoxResult<Expression> {
        let expr = self.or()?;
        if self.peek_type() == TokenType::Equal {
            if let Variable(name) = expr {
                self.advance();
                let value = self.assignment()?;
                Ok(Assign(name, Box::new(value)))
            } else {
                Err(ParseError::build(
                    self.peek(),
                    "invalid assignment target".to_string(),
                ))
            }
        } else {
            Ok(expr)
        }
    }

    fn or(&mut self) -> LoxResult<Expression> {
        let mut expr = self.and()?;
        while let TokenType::Or = self.peek_type() {
            self.advance();
            let op = self.previous().unwrap();
            let right = self.and()?;
            expr = Logical {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn and(&mut self) -> LoxResult<Expression> {
        let mut expr = self.equality()?;
        while let TokenType::And = self.peek_type() {
            self.advance();
            let op = self.previous().unwrap();
            let right = self.equality()?;
            expr = Logical {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            }
        }
        Ok(expr)
    }

    fn equality(&mut self) -> LoxResult<Expression> {
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

    fn comparison(&mut self) -> LoxResult<Expression> {
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

    fn term(&mut self) -> LoxResult<Expression> {
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

    fn factor(&mut self) -> LoxResult<Expression> {
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

    fn unary(&mut self) -> LoxResult<Expression> {
        if matches!(self.peek_type(), TokenType::Minus | TokenType::Bang) {
            self.advance();
            let op = self.previous().unwrap();
            let right = self.unary()?;
            Ok(Unary {
                op,
                right: Box::new(right),
            })
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> LoxResult<Expression> {
        let callee = self.primary()?;

        // a function can return another function
        let res = {
            if self.peek_type() == TokenType::LeftParen {
                self.advance();
                self.finish_call(callee)?
            } else {
                callee
            }
        };

        Ok(res)
    }

    fn finish_call(&mut self, callee: Expression) -> LoxResult<Expression> {
        let mut arguments = vec![];
        if self.peek_type() != TokenType::RightParen {
            arguments.push(self.expression()?);
            while self.peek_type() == TokenType::Comma {
                self.advance();
                if arguments.len() >= 255 {
                    return Err(ParseError::build(
                        self.peek(),
                        "can't have more than 255 arguments".to_string(),
                    ));
                }
                arguments.push(self.expression()?);
            }
        }
        self.consume(
            TokenType::RightParen,
            "expected `)` after arguments".to_string(),
        )?;
        Ok(Call {
            callee: Box::new(callee),
            arguments,
        })
    }

    fn primary(&mut self) -> LoxResult<Expression> {
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
            _ => Err(ParseError::build(
                self.peek(),
                "unexpected token while parsing".to_string(),
            )),
        }
    }

    fn consume(&mut self, token_type: TokenType, message: String) -> LoxResult<Token> {
        if self.peek_type() == token_type {
            self.advance();
            Ok(self.peek())
        } else {
            Err(ParseError::build(self.peek(), message))
        }
    }

    fn consume_identifier(&mut self, message: String) -> LoxResult<String> {
        if let TokenType::Identifier(name) = self.peek_type() {
            self.advance();
            Ok(name)
        } else {
            Err(ParseError::build(self.peek(), message))
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
                | TokenType::Fn
                | TokenType::Let
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
