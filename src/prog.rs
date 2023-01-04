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

#[derive(Debug, PartialEq)]
pub enum Scope {
    Global(RefCell<HashMap<String, Val>>),
    Local {
        values: RefCell<HashMap<String, Val>>,
        outer: Rc<Self>,
        global: Rc<Self>,
    },
}

impl Scope {
    pub fn new_global(globals: HashMap<String, Val>) -> Rc<Self> {
        Rc::new(Self::Global(RefCell::new(globals)))
    }

    pub fn new_local(outer: &Rc<Self>) -> Rc<Self> {
        Rc::new(Self::Local {
            values: RefCell::new(HashMap::new()),
            outer: Rc::clone(outer),
            global: Rc::clone(outer.get_global()),
        })
    }

    fn get_global<'a>(self: &'a Rc<Self>) -> &'a Rc<Self> {
        match self.as_ref() {
            Self::Global(_) => self,
            Self::Local { global, .. } => global,
        }
    }

    fn get_outer(&self) -> Option<&Rc<Self>> {
        match self {
            Self::Global(_) => None,
            Self::Local { outer, .. } => Some(outer),
        }
    }

    fn get_values(&self) -> &'_ RefCell<HashMap<String, Val>> {
        match self {
            Self::Global(values) | Self::Local { values, .. } => values,
        }
    }

    fn get_ancestor<'a>(self: &'a Rc<Self>, dist: isize) -> &'a Rc<Self> {
        let mut env = self;
        for _ in 0..dist {
            env = env.get_outer().expect("Resolver must set a valid depth")
        }
        env
    }

    pub fn def(&self, name: &str, val: Val) {
        self.get_values().borrow_mut().insert(name.to_string(), val);
    }

    pub fn get(self: &Rc<Self>, var: &Variable) -> Result<Val, ()> {
        let env = if var.depth < 0 {
            self.get_global()
        } else {
            self.get_ancestor(var.depth)
        };

        if let Some(val) = env.get_values().borrow().get(&*var.name) {
            Ok(val.clone())
        } else {
            println!("Undefined variable '{}'", var.name);
            Err(())
        }
    }

    pub fn asgn(self: &Rc<Self>, var: &Variable, new: Val) -> Result<(), ()> {
        let env = if var.depth < 0 {
            self.get_global()
        } else {
            self.get_ancestor(var.depth)
        };

        if let Some(val) = env.get_values().borrow_mut().get_mut(&*var.name) {
            *val = new;
            Ok(())
        } else {
            println!("Undefined variable '{}'", var.name);
            Err(())
        }
    }
}
