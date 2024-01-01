use crate::grammar::{Expression, Object, RuntimeError, RuntimeResult};
use std::collections::{HashMap, VecDeque};

#[derive(Debug)]
pub struct Environment(VecDeque<HashMap<String, Option<Object>>>);

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    pub fn new() -> Self {
        Self(VecDeque::from_iter([HashMap::new()]))
    }

    pub fn define(&mut self, name: &str, value: Option<Object>) {
        self.0
            .front_mut()
            .expect("no environment were found")
            .insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> RuntimeResult<&Option<Object>> {
        for env in &self.0 {
            if let Some(obj) = env.get(name) {
                return Ok(obj);
            }
        }

        Err(RuntimeError::new(format!("name `{name}` is not defined")))
    }

    pub fn enter_block(&mut self) {
        self.0.push_front(HashMap::new());
    }

    pub fn exit_block(&mut self) {
        self.0.pop_front();
    }
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Var {
        name: String,
        initializer: Option<Expression>,
    },
    Print(Expression),
    Expr(Expression),
    Block(Vec<Stmt>),
}

pub struct Interpreter(Vec<Stmt>);

impl Interpreter {
    pub fn new(statements: Vec<Stmt>) -> Self {
        Self(statements)
    }

    #[allow(clippy::only_used_in_recursion)]
    pub fn execute(&mut self, statement: &Stmt, env: &mut Environment) -> RuntimeResult<()> {
        match &statement {
            Stmt::Var { name, initializer } => {
                let eval = initializer
                    .as_ref()
                    .map(|expr| expr.evaluate(env))
                    .transpose()?;
                env.define(name, eval);
            }
            Stmt::Block(v) => {
                env.enter_block();
                for s in v {
                    self.execute(s, env)?;
                }
                env.exit_block();
            }
            Stmt::Print(expression) => println!("{}", expression.evaluate(env)?),
            Stmt::Expr(expression) => {
                expression.evaluate(env)?;
            }
        }
        Ok(())
    }

    pub fn interpret(&mut self, env: &mut Environment) {
        for statement in self.0.clone() {
            let eval = self.execute(&statement, env);
            if let Err(e) = eval {
                eprintln!("{e}");
            }
        }
    }
}
