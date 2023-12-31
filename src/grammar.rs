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

pub trait Expression {
    fn print(&self) -> String;
    fn rpn(&self) -> String;
}

impl Expression for Literal {
    fn print(&self) -> String {
        match &self.value {
            Object::String(s) => s.to_owned(),
            Object::Bool(b) => b.to_string(),
            Object::Number(x) => x.to_string(),
            Object::Nil => "nil".to_string(),
        }
    }

    fn rpn(&self) -> String {
        self.print()
    }
}

impl<E: Expression> Expression for Unary<E> {
    fn print(&self) -> String {
        format!("({} {})", self.op, self.right.print())
    }

    fn rpn(&self) -> String {
        format!("{}{}", self.op, self.right.rpn())
    }
}

impl<E1: Expression, E2: Expression> Expression for Binary<E1, E2> {
    fn print(&self) -> String {
        format!("({} {} {})", self.op, self.left.print(), self.right.print())
    }

    fn rpn(&self) -> String {
        format!("{} {} {}", self.left.rpn(), self.right.rpn(), self.op)
    }
}

impl<E: Expression> Expression for Grouping<E> {
    fn print(&self) -> String {
        format!("(group {})", self.expression.print())
    }

    fn rpn(&self) -> String {
        self.expression.rpn()
    }
}
