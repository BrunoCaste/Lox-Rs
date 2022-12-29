use std::collections::HashMap;

use crate::{stmt::Stmt, val::Val};

pub struct Prog(pub Vec<Stmt>);

impl Prog {
    pub fn exec(&self, scope: &mut Scope) -> Result<(), ()> {
        for s in &self.0 {
            s.exec(scope)?;
        }
        Ok(())
    }
}

pub struct Scope {
    environs: Vec<HashMap<String, Val>>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            environs: vec![HashMap::new()],
        }
    }

    pub fn add_inner(&mut self) {
        self.environs.push(HashMap::new());
    }

    pub fn exit_inner(&mut self) {
        self.environs.pop().unwrap();
    }

    pub fn def(&mut self, name: &str, val: Val) {
        self.environs
            .last_mut()
            .unwrap()
            .insert(name.to_string(), val);
    }

    pub fn get(&self, name: &str) -> Result<Val, ()> {
        let Some(val) = self
            .environs
            .iter()
            .rev()
            .find_map(|env| env.get(name)) else {
                    println!("Undefined variable: {name}" );
                    return Err(());
        };
        Ok(val.clone())
    }

    pub fn asgn(&mut self, name: &str, new: Val) -> Result<(), ()> {
        let Some(old) = self
            .environs
            .iter_mut()
            .rev()
            .find_map(|env| env.get_mut(name)) else {
                    println!("Undefined variable: {name}");
                    return Err(());
        };
        *old = new;
        Ok(())
    }
}
