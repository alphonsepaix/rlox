use crate::errors::{LoxError, LoxResult, RuntimeError};
use crate::expression::Object;
use crate::interpreter::{Environment, Interpreter, Signal};
use crate::parser::Stmt;
use rand::{thread_rng, Rng};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::process;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

pub enum CallableType {
    Function,
    Class,
    Instance,
}

impl Display for CallableType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CallableType::Function => write!(f, "fn"),
            CallableType::Class => write!(f, "class"),
            CallableType::Instance => write!(f, "instance"),
        }
    }
}

pub trait Callable {
    fn call(&self, objects: Vec<Object>, env: &mut Environment) -> LoxResult<Object>;

    fn arity(&self) -> usize;

    fn name(&self) -> &str;

    fn doc(&self) -> &str {
        "No documentation available."
    }

    fn r#type(&self) -> CallableType;

    fn get(&self, _name: &str) -> LoxResult<Object> {
        Err(RuntimeError::build(
            "only instances have porperties".to_string(),
        ))
    }

    fn set(&mut self, _name: &str, _value: Object) -> LoxResult<()> {
        Err(RuntimeError::build(
            "only instances can set porperties".to_string(),
        ))
    }
}

pub struct Exit;

impl Callable for Exit {
    fn call(&self, objects: Vec<Object>, _env: &mut Environment) -> LoxResult<Object> {
        let value = objects.first().expect("expected one argument");
        if let Object::Number(x) = value {
            if x.fract() == 0.0 {
                let exit_code = *x as i32;
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
        "exit"
    }

    fn doc(&self) -> &str {
        "Terminates the current process with the specified exit code."
    }

    fn r#type(&self) -> CallableType {
        CallableType::Function
    }
}

pub struct Quit;

impl Callable for Quit {
    fn call(&self, _objects: Vec<Object>, _env: &mut Environment) -> LoxResult<Object> {
        process::exit(0);
    }

    fn arity(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "quit"
    }

    fn doc(&self) -> &str {
        "Terminates the current process with an exit code of 0."
    }

    fn r#type(&self) -> CallableType {
        CallableType::Function
    }
}

pub struct Clock;

impl Callable for Clock {
    fn call(&self, _objects: Vec<Object>, _env: &mut Environment) -> LoxResult<Object> {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(time) => Ok(Object::Number(time.as_secs_f64())),
            Err(_) => Err(LoxError::Internal("could not get system time".to_string())),
        }
    }

    fn arity(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "clock"
    }

    fn doc(&self) -> &str {
        "Returns the amount of time elapsed since the Unix epoch."
    }

    fn r#type(&self) -> CallableType {
        CallableType::Function
    }
}

pub struct Type;

impl Callable for Type {
    fn call(&self, objects: Vec<Object>, _env: &mut Environment) -> LoxResult<Object> {
        let value = objects.first().expect("expected one argument");
        println!("{}", value.r#type());
        Ok(Object::Nil)
    }

    fn arity(&self) -> usize {
        1
    }
    fn name(&self) -> &str {
        "type"
    }

    fn doc(&self) -> &str {
        "Prints the type of the given object."
    }

    fn r#type(&self) -> CallableType {
        CallableType::Function
    }
}

pub struct Help;

impl Callable for Help {
    fn call(&self, objects: Vec<Object>, _env: &mut Environment) -> LoxResult<Object> {
        let value = objects.first().expect("expected one argument");
        match value {
            Object::Callable(f) => println!("{}\n\t{}", f.borrow().name(), f.borrow().doc()),
            _ => println!("No documentation available"),
        }
        Ok(Object::Nil)
    }

    fn arity(&self) -> usize {
        1
    }
    fn name(&self) -> &str {
        "help"
    }

    fn doc(&self) -> &str {
        "Prints the documentation of the given object."
    }

    fn r#type(&self) -> CallableType {
        CallableType::Function
    }
}

pub struct Rand;

impl Callable for Rand {
    fn call(&self, _objects: Vec<Object>, _env: &mut Environment) -> LoxResult<Object> {
        let x = thread_rng().gen_range(0.0..=1.0);
        Ok(Object::Number(x))
    }

    fn arity(&self) -> usize {
        0
    }
    fn name(&self) -> &str {
        "rand"
    }

    fn doc(&self) -> &str {
        "Returns a number between 0 and 1."
    }

    fn r#type(&self) -> CallableType {
        CallableType::Function
    }
}

pub struct Randint;

impl Callable for Randint {
    fn call(&self, objects: Vec<Object>, _env: &mut Environment) -> LoxResult<Object> {
        let mut iter = objects.into_iter();
        let low = iter.next().expect("expected low bound");
        let high = iter.next().expect("expected high bound");
        if let (Object::Number(low), Object::Number(high)) = (low, high) {
            if low.fract() == 0.0 && high.fract() == 0.0 {
                let low = low as i32;
                let high = high as i32;
                let rand = thread_rng().gen_range(low..=high);
                return Ok(Object::Number(rand as f64));
            }
        }
        Err(RuntimeError::build(
            "randint: expected two integers arguments".to_string(),
        ))
    }

    fn arity(&self) -> usize {
        2
    }
    fn name(&self) -> &str {
        "randint"
    }

    fn doc(&self) -> &str {
        "Returns a integer between the two provided arguments."
    }

    fn r#type(&self) -> CallableType {
        CallableType::Function
    }
}

pub struct Round;

impl Callable for Round {
    fn call(&self, objects: Vec<Object>, _env: &mut Environment) -> LoxResult<Object> {
        let mut iter = objects.into_iter();
        let value = iter.next().expect("expected a number");
        let precision = iter.next().expect("expected an integer");
        if let (Object::Number(value), Object::Number(precision)) = (value, precision) {
            if precision.fract() == 0.0 {
                let precision = precision as u32;
                let pow = u32::pow(10, precision) as f64;
                let res = (value * pow).round() / pow;
                Ok(Object::Number(res))
            } else {
                Err(RuntimeError::build(
                    "round: precision must be an integer".to_string(),
                ))
            }
        } else {
            Err(RuntimeError::build(
                "round: arguments one number and one integer".to_string(),
            ))
        }
    }

    fn arity(&self) -> usize {
        2
    }
    fn name(&self) -> &str {
        "round"
    }

    fn doc(&self) -> &str {
        "Rounds a number to a given precision in decimal digits."
    }

    fn r#type(&self) -> CallableType {
        CallableType::Function
    }
}

pub struct Print;

impl Callable for Print {
    fn call(&self, objects: Vec<Object>, _env: &mut Environment) -> LoxResult<Object> {
        let value = objects.first().expect("expected one argument");
        println!("{value}");
        Ok(Object::Nil)
    }

    fn arity(&self) -> usize {
        1
    }

    fn name(&self) -> &str {
        "print"
    }

    fn doc(&self) -> &str {
        "Prints its argument to the standard output, with a newline."
    }

    fn r#type(&self) -> CallableType {
        CallableType::Function
    }
}

pub struct Dir;

impl Callable for Dir {
    fn call(&self, _objects: Vec<Object>, env: &mut Environment) -> LoxResult<Object> {
        env.last_mut()
            .iter()
            .for_each(|(name, _)| println!("{name}"));
        Ok(Object::Nil)
    }

    fn arity(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        "dir"
    }

    fn doc(&self) -> &str {
        "Prints all the names in the current scope."
    }

    fn r#type(&self) -> CallableType {
        CallableType::Function
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
    fn call(&self, objects: Vec<Object>, env: &mut Environment) -> LoxResult<Object> {
        env.enter_block();
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

    fn r#type(&self) -> CallableType {
        CallableType::Function
    }
}

#[derive(Clone)]
pub struct UserDefinedStruct {
    name: String,
}

impl UserDefinedStruct {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Callable for UserDefinedStruct {
    fn call(&self, _objects: Vec<Object>, _env: &mut Environment) -> LoxResult<Object> {
        Ok(Object::Callable(Rc::new(RefCell::new(Instance::new(
            self.clone(),
        )))))
    }

    fn arity(&self) -> usize {
        0
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn doc(&self) -> &str {
        "No documentation available."
    }

    fn r#type(&self) -> CallableType {
        CallableType::Class
    }
}

pub struct Instance {
    base: UserDefinedStruct,
    fields: HashMap<String, Object>,
}

impl Instance {
    pub fn new(base: UserDefinedStruct) -> Instance {
        Self {
            base,
            fields: HashMap::new(),
        }
    }
}

impl Callable for Instance {
    fn call(&self, _objects: Vec<Object>, _env: &mut Environment) -> LoxResult<Object> {
        todo!();
    }

    fn arity(&self) -> usize {
        0
    }
    fn name(&self) -> &str {
        self.base.name()
    }

    fn doc(&self) -> &str {
        "A class instance."
    }

    fn r#type(&self) -> CallableType {
        CallableType::Instance
    }

    fn get(&self, name: &str) -> LoxResult<Object> {
        match self.fields.get(name) {
            None => Err(RuntimeError::build(format!(
                "undefined property `{}`",
                name
            ))),
            Some(obj) => Ok(obj.clone()),
        }
    }

    fn set(&mut self, name: &str, value: Object) -> LoxResult<()> {
        self.fields.insert(name.to_owned(), value);
        Ok(())
    }
}

impl Display for dyn Callable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{} {}>", self.r#type(), self.name())
    }
}
