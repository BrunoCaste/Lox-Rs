use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{stmt::Stmt, val::Val};

pub struct Prog(pub Vec<Stmt>);

impl Prog {
    pub fn exec(&self, scope: Rc<Scope>) -> Result<(), ()> {
        for s in &self.0 {
            s.exec(Rc::clone(&scope))?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct Scope {
    values: RefCell<HashMap<String, Val>>,
    outer: Option<Rc<Self>>,
}

impl Scope {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            values: RefCell::new(HashMap::new()),
            outer: None,
        })
    }

    pub fn from_globals(globals: HashMap<String, Val>) -> Rc<Self> {
        Rc::new(Self {
            values: RefCell::new(globals),
            outer: None,
        })
    }

    pub fn inner(outer: &Rc<Self>) -> Rc<Self> {
        Rc::new(Self {
            values: RefCell::new(HashMap::new()),
            outer: Some(Rc::clone(outer)),
        })
    }

    pub fn def(&self, name: &str, val: Val) {
        self.values.borrow_mut().insert(name.to_string(), val);
    }

    pub fn get(&self, name: &str) -> Result<Val, ()> {
        if let Some(val) = self.values.borrow().get(name) {
            Ok(val.clone())
        } else {
            self.outer.as_ref().map_or_else(
                || {
                    println!("Undefined variable '{name}'");
                    Err(())
                },
                |o| o.get(name),
            )
        }
    }

    pub fn asgn(&self, name: &str, new: Val) -> Result<(), ()> {
        if let Some(val) = self.values.borrow_mut().get_mut(name) {
            *val = new;
            Ok(())
        } else {
            self.outer.as_ref().map_or(Err(()), |o| o.asgn(name, new))
        }
    }
}
