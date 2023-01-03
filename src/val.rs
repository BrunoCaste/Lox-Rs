use std::rc::Rc;

use crate::{prog::Scope, stmt::Stmt};

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
            Func(Function::UserDef(..)) => write!(f, "<user fn>"),
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
    UserDef(Rc<Stmt>, Rc<Scope>),
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Native(a, f), Self::Native(b, g)) => a == b && f == g,
            (Self::UserDef(a, f), Self::UserDef(b, g)) => a == b && Rc::ptr_eq(f, g),
            _ => false,
        }
    }
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
            Self::UserDef(decl, closure) => match Rc::as_ref(decl) {
                Stmt::Func(_, params, body) => {
                    if params.len() != args.len() {
                        println!("Expected {} arguments, got {}", params.len(), args.len());
                        Err(())
                    } else {
                        let inner = Scope::inner(closure);
                        for (p, a) in params.iter().zip(args) {
                            inner.def(p, a);
                        }
                        body.exec(inner)
                    }
                }
                _ => unreachable!(),
            },
        }
    }
}
