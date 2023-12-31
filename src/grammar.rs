// expression     → literal
//                | unary
//                | binary
//                | grouping ;
//
// literal        → NUMBER | STRING | "true" | "false" | "nil" ;
// grouping       → "(" expression ")" ;
// unary          → ( "-" | "!" ) expression ;
// binary         → expression operator expression ;
// operator       → "==" | "!=" | "<" | "<=" | ">" | ">="
//                | "+"  | "-"  | "*" | "/" ;

use crate::scanner::{Token, TokenType};
use colored::Colorize;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::mem::discriminant;

#[derive(Debug)]
pub struct RuntimeError {
    message: String,
}

impl RuntimeError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl Error for RuntimeError {}

pub type EvaluationResult = Result<Object, RuntimeError>;

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", "runtime error:".red(), self.message)
    }
}
#[derive(PartialEq, Clone)]
pub enum Object {
    Str(String),
    Number(f64),
    Bool(bool),
    Nil,
}
use Object::*;

impl Object {
    fn truthy(&self) -> bool {
        matches!(self, Bool(false) | Nil)
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Str(s) => write!(f, "\"{s}\""),
            Number(x) => write!(f, "{x}"),
            Bool(b) => write!(f, "{b}"),
            Nil => write!(f, "nil"),
        }
    }
}

pub enum Expression {
    Literal(Object),
    Unary {
        op: Token,
        right: Box<Expression>,
    },
    Binary {
        left: Box<Expression>,
        op: Token,
        right: Box<Expression>,
    },
    Grouping(Box<Expression>),
}
use Expression::*;

impl Expression {
    pub fn evaluate(&self) -> EvaluationResult {
        match self {
            Literal(object) => Ok(object.clone()),
            Unary { op, right } => {
                let right = right.evaluate()?;
                match &op.r#type {
                    TokenType::Bang => Ok(Bool(right.truthy())),
                    TokenType::Minus => {
                        if let Number(x) = right {
                            Ok(Number(-x))
                        } else {
                            Err(RuntimeError::new(
                                "unary operator `-` only works with numbers".to_string(),
                            ))
                        }
                    }
                    token => Err(RuntimeError::new(format!(
                        "invalid token for unary expression: `{:?}`",
                        token
                    ))),
                }
            }
            Binary { left, op, right } => {
                let left = left.evaluate()?;
                let right = right.evaluate()?;
                if discriminant(&left) != discriminant(&right) {
                    return Err(RuntimeError::new(
                        "both operands must be of the same type".to_string(),
                    ));
                }
                match (left, &op.r#type, right) {
                    (left, TokenType::EqualEqual, right) => Ok(Bool(left == right)),
                    (left, TokenType::BangEqual, right) => Ok(Bool(left != right)),
                    (Number(x), op, Number(y)) => match &op {
                        TokenType::Plus => Ok(Number(x + y)),
                        TokenType::Minus => Ok(Number(x - y)),
                        TokenType::Slash => {
                            if y == 0.0 {
                                Err(RuntimeError::new("division by zero".to_string()))
                            } else {
                                Ok(Number(x / y))
                            }
                        }
                        TokenType::Star => Ok(Number(x * y)),
                        TokenType::Greater => Ok(Bool(x > y)),
                        TokenType::GreaterEqual => Ok(Bool(x >= y)),
                        TokenType::Less => Ok(Bool(x < y)),
                        TokenType::LessEqual => Ok(Bool(x <= y)),
                        op => Err(RuntimeError::new(format!(
                            "unsupported operation between numbers: `{:?}`",
                            op
                        ))),
                    },
                    (Str(s1), op, Str(s2)) => match &op {
                        TokenType::Plus => Ok(Str(s1.to_owned() + &s2)),
                        TokenType::Greater => Ok(Bool(s1 > s2)),
                        TokenType::GreaterEqual => Ok(Bool(s1 >= s2)),
                        TokenType::Less => Ok(Bool(s1 < s2)),
                        TokenType::LessEqual => Ok(Bool(s1 <= s2)),
                        op => Err(RuntimeError::new(format!(
                            "unsupported operation between strings: `{:?}`",
                            op
                        ))),
                    },
                    _ => Err(RuntimeError::new("can't evaluate expression".to_string())),
                }
            }
            Grouping(expr) => expr.evaluate(),
        }
    }

    pub fn repr(&self) -> String {
        match self {
            Literal(object) => object.to_string(),
            Unary { op, right } => format!("({} {})", op, right.repr()),
            Binary { left, op, right } => {
                format!("({} {} {})", op, left.repr(), right.repr())
            }
            Grouping(expression) => format!("(group {})", expression.repr()),
        }
    }

    pub fn rpn(&self) -> String {
        match self {
            Literal(_) => self.repr(),
            Unary { op, right } => format!("{}{}", op, right.rpn()),
            Binary { left, op, right } => {
                format!("{} {} {}", left.rpn(), right.rpn(), op)
            }
            Grouping(expression) => expression.rpn(),
        }
    }
}
