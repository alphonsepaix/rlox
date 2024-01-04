use crate::errors::{LoxError, LoxResult, RuntimeError};
use crate::expression::{Expression, Object};
use crate::interpreter::{Environment, Interpreter, Signal};
use crate::parser::Stmt;
use std::fmt::{Display, Formatter};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

pub trait Callable {
    fn call(&self, arguments: &[Expression], env: &mut Environment) -> LoxResult<Object>;

    fn arity(&self) -> usize;

    fn name(&self) -> &str;

    fn doc(&self) -> &str {
        "No documentation available."
    }
}

pub struct Exit;

impl Callable for Exit {
    fn call(&self, arguments: &[Expression], env: &mut Environment) -> LoxResult<Object> {
        let arg = arguments.first().expect("expected one argument");
        let value = arg.evaluate(env)?;
        if let Object::Number(x) = value {
            if x.fract() == 0.0 {
                let exit_code = x as i32;
                process::exit(exit_code);
            }
        }
        Err(RuntimeError::build(format!(
            "{}: expected an integer",
            self.name()
        )))
    }

    fn arity(&self) -> usize {
        1
    }

    fn name(&self) -> &str {
        "<fn exit>"
    }

    fn doc(&self) -> &str {
        "Terminates the current process with the specified exit code."
    }
}

pub struct Quit;

impl Callable for Quit {
    fn call(&self, _arguments: &[Expression], _env: &mut Environment) -> LoxResult<Object> {
        process::exit(0);
    }

    fn arity(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "<fn quit>"
    }

    fn doc(&self) -> &str {
        "Terminates the current process with an exit code of 0."
    }
}

pub struct Clock;

impl Callable for Clock {
    fn call(&self, _arguments: &[Expression], _env: &mut Environment) -> LoxResult<Object> {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(time) => Ok(Object::Number(time.as_secs_f64())),
            Err(_) => Err(LoxError::Internal("could not get system time".to_string())),
        }
    }

    fn arity(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "<fn clock>"
    }

    fn doc(&self) -> &str {
        "Returns the amount of time elapsed since the Unix epoch."
    }
}

pub struct Help;

impl Callable for Help {
    fn call(&self, arguments: &[Expression], env: &mut Environment) -> LoxResult<Object> {
        let arg = arguments.first().expect("expected one argument");
        let value = arg.evaluate(env)?;
        match value {
            Object::Str(_) => println!("<string> object"),
            Object::Number(_) => println!("<f64> object"),
            Object::Bool(_) => println!("<bool> object"),
            Object::Callable(f) => println!("{}\n\t{}", f.name(), f.doc()),
            Object::Nil => println!("<nil> object"),
        }
        Ok(Object::Nil)
    }

    fn arity(&self) -> usize {
        1
    }
    fn name(&self) -> &str {
        "<fn help>"
    }

    fn doc(&self) -> &str {
        "Prints the documentation of the given object."
    }
}

pub struct Print;

impl Callable for Print {
    fn call(&self, arguments: &[Expression], env: &mut Environment) -> LoxResult<Object> {
        let arg = arguments.first().expect("expected one argument");
        let value = arg.evaluate(env)?;
        println!("{value}");
        Ok(Object::Nil)
    }

    fn arity(&self) -> usize {
        1
    }

    fn name(&self) -> &str {
        "<fn print>"
    }

    fn doc(&self) -> &str {
        "Prints its argument to the standard output, with a newline."
    }
}

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

impl Callable for UserDefinedFunction {
    fn call(&self, arguments: &[Expression], env: &mut Environment) -> LoxResult<Object> {
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

    fn name(&self) -> &str {
        &self.name
    }
}

impl Display for dyn Callable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
