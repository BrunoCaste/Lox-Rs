use std::rc::Rc;
#[derive(PartialEq, Debug, Clone)]
pub enum Val {
    NoVal,
    Number(f64),
    Boolean(bool),
    String(Rc<str>),
    Nil,
}

impl std::fmt::Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Val::*;
        match self {
            Number(x) => write!(f, "{x}"),
            Boolean(b) => write!(f, "{b}"),
            String(s) => write!(f, "{s}"),
            Nil => write!(f, "nil"),
            NoVal => write!(f, "???"),
        }
    }
}

impl From<Val> for bool {
    fn from(value: Val) -> Self {
        !matches!(value, Val::Nil | Val::Boolean(false))
    }
}
