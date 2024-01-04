use crate::errors::{LoxResult, RuntimeError};
use crate::expression::{Expression, Object};
use crate::interpreter::{Environment, Interpreter, Signal};
use crate::parser::Stmt;
use std::fmt::{Display, Formatter};

pub struct UserDefinedFunction {
    name: String,
    body: Vec<Stmt>,
    parameters: Vec<String>,
}

impl UserDefinedFunction {
    pub fn new(name: String, body: Vec<Stmt>, parameters: Vec<String>) -> Self {
        Self {
            name,
            body,
            parameters,
        }
    }
}

pub trait Callable: Display {
    fn call(
        &self,
        name: &str,
        arguments: &[Expression],
        env: &mut Environment,
    ) -> LoxResult<Object>;

    fn arity(&self) -> usize;
}

impl Callable for UserDefinedFunction {
    fn call(
        &self,
        name: &str,
        arguments: &[Expression],
        env: &mut Environment,
    ) -> LoxResult<Object> {
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
        self.parameters
            .iter()
            .zip(objects)
            .for_each(|(param, value)| env.define(param, Some(value)));
        let interpreter = Interpreter::new();
        let mut return_value = Object::Nil;
        if let Some(Signal::Return(Some(expr))) = interpreter.interpret(env, &self.body)? {
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

    fn arity(&self) -> usize {
        self.parameters.len()
    }
}

impl Display for UserDefinedFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.name)
    }
}
