use crate::errors::{LoxResult, RuntimeError};
use crate::interpreter::{Environment, Interpreter, Signal};
use crate::scanner::{Token, TokenType};
use std::fmt::{Display, Formatter};
use Expression::*;

#[derive(PartialEq, Clone, Debug)]
pub enum Object {
    Str(String),
    Number(f64),
    Bool(bool),
    Function {
        name: String,
        body: Vec<Stmt>,
        parameters: Vec<String>,
    },
    Nil,
}

use crate::parser::Stmt;

impl Object {
    pub fn truthy(&self) -> bool {
        !matches!(self, Object::Bool(false) | Object::Nil)
    }

    fn is_equal(&self, other: &Object) -> bool {
        if self == &Object::Nil && other == &Object::Nil {
            return true;
        }
        if self == &Object::Nil {
            return false;
        }
        self == other
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
            Function { .. } => write!(f, "<fn>"),
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
}

impl Expression {
    pub fn evaluate(&self, env: &mut Environment) -> LoxResult<Object> {
        use Object::*;
        match self {
            Literal(object) => Ok(object.clone()),
            Unary { op, right } => {
                let right = right.evaluate(env)?;
                match &op.r#type {
                    TokenType::Bang => Ok(Bool(!right.truthy())),
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
                    (left, TokenType::EqualEqual, right) => Ok(Bool(left.is_equal(&right))),
                    (left, TokenType::BangEqual, right) => Ok(Bool(!left.is_equal(&right))),
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
                    (Str(s1), TokenType::Plus, right) => {
                        Ok(Str(format!("{}{}", s1, right.to_string())))
                    }
                    (left, TokenType::Plus, Str(s2)) => {
                        Ok(Str(format!("{}{}", left.to_string(), s2)))
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
                    if left.truthy() {
                        return Ok(Bool(true));
                    }
                } else if !left.truthy() {
                    return Ok(Bool(false));
                }
                Ok(Bool(right.evaluate(env)?.truthy()))
            }
            Call { callee, arguments } => {
                // callee is a Variable, get the object living in the env
                let name = callee.to_string();
                let callee = callee.evaluate(env)?;
                callee.call(&name, arguments, env)
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
        };
        write!(f, "{s}")
    }
}

trait Callable {
    fn call(
        &self,
        name: &str,
        arguments: &[Expression],
        env: &mut Environment,
    ) -> LoxResult<Object>;

    fn arity(&self) -> usize;
}

impl Callable for Object {
    fn call(
        &self,
        name: &str,
        arguments: &[Expression],
        env: &mut Environment,
    ) -> LoxResult<Object> {
        match self {
            Object::Function {
                name,
                body,
                parameters,
            } => {
                let arity = self.arity();
                let num_args = arguments.len();
                if num_args != arity {
                    return Err(RuntimeError::build(format!(
                        "`{name}`: expected {arity} argument{} but got {num_args}",
                        if arity > 1 { 's' } else { '\0' },
                    )));
                }
                env.enter_block();
                let objects = arguments
                    .iter()
                    .map(|arg| arg.evaluate(env))
                    .collect::<Result<Vec<_>, _>>()?;
                parameters
                    .iter()
                    .zip(objects)
                    .for_each(|(param, value)| env.define(param, Some(value)));
                let interpreter = Interpreter::new();
                let mut return_value = Object::Nil;
                if let Some(Signal::Return(Some(expr))) = interpreter.interpret(env, &body)? {
                    let eval = expr.evaluate(env);
                    return_value = match eval {
                        Ok(obj) => obj,
                        Err(e) => {
                            env.exit_block();
                            return Err(e);
                        }
                    };
                }
                env.exit_block();
                Ok(return_value)
            }
            _ => Err(RuntimeError::build(format!("`{name}` is not callable"))),
        }
    }

    fn arity(&self) -> usize {
        match self {
            Object::Function { parameters, .. } => parameters.len(),
            _ => panic!("not a function"),
        }
    }
}