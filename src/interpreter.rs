use crate::grammar::{Object, RuntimeError, RuntimeResult, Stmt};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Environment(HashMap<String, Object>);

impl Environment {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    pub fn define(&mut self, name: &str, value: Object) {
        self.0.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> RuntimeResult<&Object> {
        match self.0.get(name) {
            None => Err(RuntimeError::new(format!("name `{name}` is not defined"))),
            Some(obj) => Ok(obj),
        }
    }
}

pub struct Interpreter(Vec<Stmt>);

impl Interpreter {
    pub fn new(statements: Vec<Stmt>) -> Self {
        Self(statements)
    }

    pub fn execute(&mut self, statement: Stmt, env: &mut Environment) -> RuntimeResult<()> {
        let eval = statement.expression().evaluate(env)?;
        match &statement {
            Stmt::Var { name, .. } => env.define(name, eval),
            _ => {
                if let Stmt::Print(_) = statement {
                    println!("{eval}");
                }
            }
        }
        Ok(())
    }

    pub fn interpret(&mut self, env: &mut Environment) {
        for statement in self.0.clone() {
            let eval = self.execute(statement, env);
            if let Err(e) = eval {
                eprintln!("{e}");
            }
        }
    }
}
