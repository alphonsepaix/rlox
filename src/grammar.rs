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

impl Expression {
    pub fn repr(&self) -> String {
        match self {
            Expression::Literal(object) => match object {
                Str(s) => s.to_owned(),
                Bool(b) => b.to_string(),
                Number(x) => x.to_string(),
                Nil => "nil".to_string(),
            },
            Expression::Unary { op, right } => format!("({} {})", op, right.repr()),
            Expression::Binary { left, op, right } => {
                format!("({} {} {})", op, left.repr(), right.repr())
            }
            Expression::Grouping(expression) => format!("(group {})", expression.repr()),
        }
    }

    pub fn rpn(&self) -> String {
        match self {
            Expression::Literal(_) => self.repr(),
            Expression::Unary { op, right } => format!("{}{}", op, right.rpn()),
            Expression::Binary { left, op, right } => {
                format!("{} {} {}", left.rpn(), right.rpn(), op)
            }
            Expression::Grouping(expression) => expression.rpn(),
        }
    }
}
