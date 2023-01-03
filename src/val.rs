use std::rc::Rc;
#[derive(PartialEq, Debug, Clone)]
pub enum Val {
    NoVal,
    Number(f64),
    Boolean(bool),
    String(Rc<str>),
    Nil,
    Func(Function),
}

impl std::fmt::Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Val::*;
        match self {
            Number(x) => write!(f, "{x}"),
            Boolean(b) => write!(f, "{b}"),
            String(s) => write!(f, "{s}"),
            Nil => write!(f, "nil"),
            Func(Function::Native(..)) => write!(f, "<native fn>"),
            NoVal => write!(f, "???"),
        }
    }
}

impl From<Val> for bool {
    fn from(value: Val) -> Self {
        !matches!(value, Val::Nil | Val::Boolean(false))
    }
}

#[derive(Debug, Clone)]
pub enum Function {
    Native(u8, fn(Vec<Val>) -> Val),
}

pub trait Callable {
    fn call(&self, args: Vec<Val>) -> Result<Val, ()>;
}

impl Callable for Function {
    fn call(&self, args: Vec<Val>) -> Result<Val, ()> {
        match self {
            Self::Native(arity, f) => {
                if *arity as usize != args.len() {
                    println!("Expected {} arguments, got {}", arity, args.len());
                    Err(())
                } else {
                    Ok(f(args))
                }
            }
        }
    }
}
