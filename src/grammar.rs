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
use std::fmt::{Display, Formatter};

pub enum Object {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
}

pub struct Literal {
    value: Object,
}

impl Literal {
    pub fn new(value: Object) -> Self {
        Self { value }
    }
}

pub struct Unary<E: Expression> {
    op: Token,
    right: E,
}

impl<E: Expression> Unary<E> {
    pub fn new(op: Token, right: E) -> Self {
        Self { op, right }
    }
}

pub struct Binary<E1: Expression, E2: Expression> {
    left: E1,
    op: Token,
    right: E2,
}

impl<E1: Expression, E2: Expression> Binary<E1, E2> {
    pub fn new(left: E1, op: Token, right: E2) -> Self {
        Self { left, op, right }
    }
}

pub struct Grouping<E: Expression> {
    expression: E,
}

impl<E: Expression> Grouping<E> {
    pub fn new(expression: E) -> Self {
        Self { expression }
    }
}

pub trait Expression: Display {}

impl Expression for Literal {}

impl<E: Expression> Expression for Unary<E> {}

impl<E1: Expression, E2: Expression> Expression for Binary<E1, E2> {}

impl<E: Expression> Expression for Grouping<E> {}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.value {
            Object::String(s) => write!(f, "{}", s),
            Object::Bool(b) => write!(f, "{}", b),
            Object::Number(x) => write!(f, "{}", x),
            Object::Nil => write!(f, "nil"),
        }
    }
}

impl<E: Expression> Display for Unary<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {})", self.op, self.right)
    }
}

impl<E1: Expression, E2: Expression> Display for Binary<E1, E2> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {})", self.op, self.left, self.right)
    }
}

impl<E: Expression> Display for Grouping<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(group {})", self.expression)
    }
}

pub trait ReversePolishNotation {
    fn rpn(&self) -> String;
}

impl ReversePolishNotation for Literal {
    fn rpn(&self) -> String {
        format!("{}", self)
    }
}

impl<E: Expression + ReversePolishNotation> ReversePolishNotation for Unary<E> {
    fn rpn(&self) -> String {
        format!("{}{}", self.op, self.right.rpn())
    }
}

impl<E1: Expression + ReversePolishNotation, E2: Expression + ReversePolishNotation>
    ReversePolishNotation for Binary<E1, E2>
{
    fn rpn(&self) -> String {
        format!("{} {} {}", self.left.rpn(), self.right.rpn(), self.op)
    }
}

impl<E: Expression + ReversePolishNotation> ReversePolishNotation for Grouping<E> {
    fn rpn(&self) -> String {
        format!("{}", self.expression.rpn())
    }
}
