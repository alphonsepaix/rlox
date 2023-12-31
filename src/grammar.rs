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

use crate::scanner::Token;

pub enum Object {
    Str(String),
    Number(f64),
    Bool(bool),
    Nil,
}
use Object::*;

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

pub trait Representation {
    fn repr(&self) -> String;
    fn rpn(&self) -> String;
}

impl Representation for Expression {
    fn repr(&self) -> String {
        match self {
            Literal(object) => match object {
                Str(s) => s.to_owned(),
                Bool(b) => b.to_string(),
                Number(x) => x.to_string(),
                Nil => "nil".to_string(),
            },
            Unary { op, right } => format!("({} {})", op, right.repr()),
            Binary { left, op, right } => {
                format!("({} {} {})", op, left.repr(), right.repr())
            }
            Grouping(expression) => format!("(group {})", expression.repr()),
        }
    }

    fn rpn(&self) -> String {
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
