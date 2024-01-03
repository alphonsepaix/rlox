use crate::errors::{RuntimeError, RuntimeResult};
use crate::grammar::Object;
use crate::parser::Stmt;
use std::collections::hash_map::Entry::Occupied;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct Environment(Vec<HashMap<String, Option<Object>>>);

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    pub fn new() -> Self {
        Self(vec![HashMap::new()])
    }

    pub fn define(&mut self, name: &str, value: Option<Object>) {
        self.0
            .last_mut()
            .expect("no environment were found")
            .insert(name.to_string(), value);
    }

    pub fn update(&mut self, name: &str, value: Object) -> RuntimeResult<()> {
        for env in self.0.iter_mut().rev() {
            if let Occupied(ref mut entry) = env.entry(name.to_string()) {
                *entry.get_mut() = Some(value.clone());
                return Ok(());
            }
        }

        Err(RuntimeError::new(format!("name `{name}` is not defined")))
    }

    pub fn get(&self, name: &str) -> RuntimeResult<&Option<Object>> {
        for env in self.0.iter().rev() {
            if let Some(obj) = env.get(name) {
                return Ok(obj);
            }
        }

        Err(RuntimeError::new(format!("name `{name}` is not defined")))
    }

    pub fn enter_block(&mut self) {
        self.0.push(HashMap::new());
    }

    pub fn exit_block(&mut self) {
        self.0.pop();
    }
}

pub enum Signal {
    Continue,
    Break,
}

impl Display for Signal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Signal::Continue => write!(f, "continue"),
            Signal::Break => write!(f, "break"),
        }
    }
}

pub struct Interpreter;

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self
    }

    #[allow(clippy::only_used_in_recursion)]
    pub fn execute(
        &self,
        statement: &Stmt,
        env: &mut Environment,
    ) -> RuntimeResult<Option<Signal>> {
        match statement {
            Stmt::Var { name, initializer } => {
                let eval = initializer
                    .as_ref()
                    .map(|expr| expr.evaluate(env))
                    .transpose()?;
                env.define(name, eval);
            }
            Stmt::Function { name, .. } => {
                env.define(name, Some(Object::Func(Box::new(statement.clone()))));
            }
            Stmt::Block(block) => {
                env.enter_block();
                for s in block {
                    let control = self.execute(s, env)?;
                    if control.is_some() {
                        env.exit_block();
                        return Ok(control);
                    }
                }
                env.exit_block();
            }
            Stmt::Print(expression) => println!("{}", expression.evaluate(env)?),
            Stmt::Expr(expression) => {
                expression.evaluate(env)?;
            }
            Stmt::If {
                condition,
                then_stmt,
                else_stmt,
            } => {
                if let Object::Bool(true) = condition.evaluate(env)? {
                    return self.execute(then_stmt, env);
                } else if let Some(else_stmt) = else_stmt {
                    return self.execute(else_stmt, env);
                }
            }
            Stmt::While {
                condition,
                body,
                increment,
            } => {
                while condition.evaluate(env)?.truthy() {
                    if let Some(signal) = self.execute(body, env)? {
                        match signal {
                            Signal::Break => break,
                            Signal::Continue => {
                                if let Some(increment) = increment {
                                    increment.evaluate(env)?;
                                }
                                continue;
                            }
                        }
                    }
                }
            }
            Stmt::Break => return Ok(Some(Signal::Break)),
            Stmt::Continue => return Ok(Some(Signal::Continue)),
        }
        Ok(None)
    }

    pub fn interpret(self, env: &mut Environment, statements: &[Stmt]) {
        for statement in statements {
            match self.execute(statement, env) {
                Err(e) => eprintln!("{e}"),
                Ok(Some(signal)) => panic!("signal `{signal}` unhandled", signal = signal),
                _ => (),
            }
        }
    }
}
