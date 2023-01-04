use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{expr::Variable, stmt::Stmt, val::Val};

pub struct Prog(pub Vec<Stmt>);

impl Prog {
    pub fn exec(&self, scope: Rc<Scope>) -> Result<(), ()> {
        for s in &self.0 {
            s.exec(Rc::clone(&scope))?;
        }
        Ok(())
    }
}
