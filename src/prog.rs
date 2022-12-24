use std::collections::HashMap;

use crate::{expr::Val, stmt::Stmt};

pub struct Scope<'a> {
    values: HashMap<String, Val>,
    fallback: Option<&'a mut Scope<'a>>,
}

impl<'a> Scope<'a> {
    pub fn new(fallback: Option<&'a mut Self>) -> Self {
        Self {
            values: HashMap::new(),
            fallback,
        }
    }
    pub fn def(&mut self, name: &String, val: Val) {
        self.values.insert(name.to_string(), val);
    }

    pub fn get(&self, name: &String) -> Result<Val, ()> {
        if let Some(val) = self.values.get(name) {
            Ok(val.clone())
        } else {
            self.fallback
                .as_ref()
                .map_or_else(|| Err(()), |f| f.get(name))
        }
    }
}
