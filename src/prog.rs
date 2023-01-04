use std::rc::Rc;

use crate::{scope::Scope, stmt::Stmt};

#[derive(Debug)]
pub struct Prog {
    pub stmts: Vec<Stmt>,
}

impl Prog {
    pub fn new() -> Self {
        Self { stmts: Vec::new() }
    }

    pub fn exec(&self, scope: Rc<Scope>) -> Result<(), ()> {
        for s in &self.stmts {
            s.exec(Rc::clone(&scope))?;
        }
        Ok(())
    }
}
