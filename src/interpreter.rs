use crate::grammar::Expression::Assign;
use crate::grammar::{Object, RuntimeError, RuntimeResult};
use crate::parser::Stmt;
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

pub enum Context {
    Repl,
    File,
}

pub struct Interpreter {
    statements: Vec<Stmt>,
    context: Context,
}

impl Interpreter {
    pub fn new(statements: Vec<Stmt>, context: Context) -> Self {
        Self {
            statements,
            context,
        }
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
                let eval = expression.evaluate(env)?;
                if matches!(self.context, Context::Repl) && !matches!(expression, Assign(..)) {
                    println!("{eval}");
                }
            }
            Stmt::If {
                condition,
                then_stmt,
                else_stmt,
            } => {
                if let Object::Bool(true) = condition.evaluate(env)? {
                    self.execute(then_stmt, env)?;
                } else if let Some(else_stmt) = else_stmt {
                    self.execute(else_stmt, env)?;
                }
            }
        }
        Ok(())
    }

    pub fn interpret(&mut self, env: &mut Environment) {
        for statement in self.statements.clone() {
            let eval = self.execute(&statement, env);
            if let Err(e) = eval {
                eprintln!("{e}");
            }
        }
    }
}
