use crate::errors::{LoxResult, RuntimeError};
use crate::functions::Callable;
use crate::interpreter::Environment;
use crate::scanner::{Token, TokenType};
use std::fmt::{Debug, Display, Formatter};
use std::ops::Not;
use std::rc::Rc;
use Expression::*;

pub enum Object {
    Str(String),
    Number(f64),
    Bool(bool),
    Callable(Rc<dyn Callable>),
    Nil,
}

impl Object {
    pub fn callable(&self) -> bool {
        matches!(self, Object::Callable(..))
    }

    pub fn r#type(&self) -> String {
        match self {
            Object::Str(_) => "<string> object".to_string(),
            Object::Number(_) => "<f64> object".to_string(),
            Object::Bool(_) => "<bool> object".to_string(),
            Object::Callable(f) => format!("<{}> object", f.r#type()),
            Object::Nil => "<nil> object".to_string(),
        }
    }
}

impl From<Object> for bool {
    fn from(value: Object) -> Self {
        !matches!(value, Object::Bool(false) | Object::Nil)
    }
}

impl Not for Object {
    type Output = bool;
    fn not(self) -> Self::Output {
        !Into::<bool>::into(self)
    }
}

impl Debug for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Str(s) => write!(f, "{s:?}"),
            Object::Number(x) => write!(f, "{x:?}"),
            Object::Bool(b) => write!(f, "{b:?}"),
            Object::Callable(c) => write!(f, "{}", c),
            Object::Nil => write!(f, "nil"),
        }
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        use Object::*;
        match (self, other) {
            (Str(s1), Str(s2)) => s1 == s2,
            (Bool(b1), Bool(b2)) => b1 == b2,
            (Number(x1), Number(x2)) => x1 == x2,
            (Nil, Nil) => true,
            _ => false,
        }
    }
}

impl Clone for Object {
    fn clone(&self) -> Self {
        match self {
            Object::Str(s) => Object::Str(s.clone()),
            Object::Number(x) => Object::Number(*x),
            Object::Bool(b) => Object::Bool(*b),
            Object::Nil => Object::Nil,
            Object::Callable(f) => Object::Callable(Rc::clone(f)),
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Object::*;

        match self {
            Str(s) => write!(f, "{s}"),
            Number(x) => write!(f, "{x}"),
            Bool(b) => write!(f, "{b}"),
            Nil => write!(f, "nil"),
            Callable(c) => write!(f, "{}", *c),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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
    Variable(String),
    Assign(String, Box<Expression>),
    Logical {
        left: Box<Expression>,
        op: Token,
        right: Box<Expression>,
    },
    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },
    Get {
        name: String,
        object: Box<Expression>,
    },
    Set {
        object: Box<Expression>,
        name: String,
        value: Box<Expression>,
    },
}

impl Expression {
    pub fn evaluate(&self, env: &mut Environment) -> LoxResult<Object> {
        use Object::*;
        match self {
            Literal(object) => Ok(object.clone()),
            Unary { op, right } => {
                let right = right.evaluate(env)?;
                match &op.r#type {
                    TokenType::Bang => Ok(Bool(right.into())),
                    TokenType::Minus => {
                        if let Number(x) = right {
                            Ok(Number(-x))
                        } else {
                            Err(RuntimeError::build(
                                "unary operator `-` only works with numbers".to_string(),
                            ))
                        }
                    }
                    token => Err(RuntimeError::build(format!(
                        "invalid token for unary expression: `{:?}`",
                        token
                    ))),
                }
            }
            Binary { left, op, right } => {
                let left = left.evaluate(env)?;
                let right = right.evaluate(env)?;
                match (left, &op.r#type, right) {
                    (left, TokenType::EqualEqual, right) => Ok(Bool(left == right)),
                    (left, TokenType::BangEqual, right) => Ok(Bool(left != right)),
                    (Number(x), op, Number(y)) => match &op {
                        TokenType::Plus => Ok(Number(x + y)),
                        TokenType::Minus => Ok(Number(x - y)),
                        TokenType::Slash => {
                            if y == 0.0 {
                                Err(RuntimeError::build("division by zero".to_string()))
                            } else {
                                Ok(Number(x / y))
                            }
                        }
                        TokenType::Star => Ok(Number(x * y)),
                        TokenType::Greater => Ok(Bool(x > y)),
                        TokenType::GreaterEqual => Ok(Bool(x >= y)),
                        TokenType::Less => Ok(Bool(x < y)),
                        TokenType::LessEqual => Ok(Bool(x <= y)),
                        op => Err(RuntimeError::build(format!(
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
                        op => Err(RuntimeError::build(format!(
                            "unsupported operation between strings: `{:?}`",
                            op
                        ))),
                    },
                    (Str(s1), TokenType::Plus, right) if right != Nil => {
                        Ok(Str(format!("{}{}", s1, right)))
                    }
                    (left, TokenType::Plus, Str(s2)) if left != Nil => {
                        Ok(Str(format!("{}{}", left, s2)))
                    }
                    _ => Err(RuntimeError::build(
                        "can't evaluate expression: unsupported operation between types"
                            .to_string(),
                    )),
                }
            }
            Grouping(expr) => expr.evaluate(env),
            Variable(name) => env
                .get(name)?
                .as_ref()
                .ok_or(RuntimeError::build(format!(
                    "variable `{name}` used uninitialized"
                )))
                .cloned(),
            Assign(name, expr) => {
                let eval = expr.evaluate(env)?;
                env.update(name, eval.clone())?;
                Ok(eval)
            }
            Logical { left, op, right } => {
                let left = left.evaluate(env)?;
                if let TokenType::Or = op.r#type {
                    if left.into() {
                        return Ok(Bool(true));
                    }
                } else if !left {
                    return Ok(Bool(false));
                }
                Ok(Bool(right.evaluate(env)?.into()))
            }
            Call { callee, arguments } => {
                // callee is a Variable, get the object living in the env
                let name = callee.to_string();
                let callee = callee.evaluate(env)?;
                if let Callable(f) = callee {
                    let arity = f.arity();
                    let num_args = arguments.len();
                    if num_args != arity {
                        return Err(RuntimeError::build(format!(
                            "`{f}`: expected {arity} argument{} but got {num_args}",
                            if arity > 1 { 's' } else { '\0' },
                        )));
                    }
                    let objects = arguments
                        .iter()
                        .map(|arg| arg.evaluate(env))
                        .collect::<Result<Vec<_>, _>>()?;
                    f.call(objects, env)
                } else {
                    Err(RuntimeError::build(format!("{name} is not callable")))
                }
            }
            Get { name, object } => {
                if let Callable(f) = object.evaluate(env)? {
                    f.get(name)
                } else {
                    Err(RuntimeError::build(format!("{name} is not callable")))
                }
            }
            Set {
                object,
                name,
                value,
            } => {
                if let Callable(f) = object.evaluate(env)? {
                    let value = value.evaluate(env)?;
                    f.set(name, value.clone())?;
                    Ok(value)
                } else {
                    Err(RuntimeError::build(format!("{name} is not callable")))
                }
            }
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Literal(object) => object.to_string(),
            Unary { op, right } => format!("({} {})", op, right),
            Binary { left, op, right } => {
                format!("({} {} {})", op, left, right)
            }
            Grouping(expression) => format!("(group {})", expression),
            Variable(name) => name.to_owned(),
            Assign(_, expression) => expression.to_string(),
            Logical { .. } => todo!(),
            Call { .. } => todo!(),
            Get { .. } => todo!(),
            Set { .. } => todo!(),
        };
        write!(f, "{s}")
    }
}
