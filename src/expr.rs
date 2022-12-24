#[derive(PartialEq, Debug, Clone)]
pub enum Val {
    Number(f64),
    Boolean(bool),
    String(String),
    Nil,
}

impl std::fmt::Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Number(x) => write!(f, "{x}"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::String(s) => write!(f, "{s}"),
            Self::Nil => write!(f, "nil"),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Expr {
    // A variant for grouping is not necessary,
    // as long as the parser handles `Paren`s correctly
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    Ne(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Ge(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Le(Box<Expr>, Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Opp(Box<Expr>),
    Lit(Val),
    Var(String),
}

macro_rules! try_numeric {
    ($lhs:ident $op:tt $rhs:ident => $var:tt) => {{
        let (x, y) = ($lhs.eval()?, $rhs.eval()?);
        match (&x, &y) {
            (Val::Number(x), Val::Number(y)) => Ok(Val::$var(x $op y)),
            (Val::Number(_), _) => Err(()),
                    (_, _) => Err(()),
        }
    }};
}

impl Expr {
    pub fn eval(&self) -> Result<Val, ()> {
        use Expr::*;
        match self {
            And(lhs, rhs) => match lhs.eval()? {
                b @ (Val::Nil | Val::Boolean(false)) => Ok(b),
                _ => rhs.eval(),
            },
            Or(lhs, rhs) => match lhs.eval()? {
                Val::Nil | Val::Boolean(false) => rhs.eval(),
                b => Ok(b),
            },
            Eq(lhs, rhs) => {
                let (x, y) = (lhs.eval()?, rhs.eval()?);
                Ok(Val::Boolean(x == y))
            }
            Ne(lhs, rhs) => {
                let (x, y) = (lhs.eval()?, rhs.eval()?);
                Ok(Val::Boolean(x != y))
            }
            Gt(lhs, rhs) => try_numeric!(lhs >  rhs => Boolean),
            Ge(lhs, rhs) => try_numeric!(lhs >= rhs => Boolean),
            Lt(lhs, rhs) => try_numeric!(lhs <  rhs => Boolean),
            Le(lhs, rhs) => try_numeric!(lhs <= rhs => Boolean),
            Add(lhs, rhs) => match (lhs.eval()?, rhs.eval()?) {
                (Val::Number(x), Val::Number(y)) => Ok(Val::Number(x + y)),
                (Val::String(s), Val::String(t)) => Ok(Val::String(s + &t)),
                _ => Err(()),
            },
            Sub(lhs, rhs) => try_numeric!(lhs - rhs => Number),
            Mul(lhs, rhs) => try_numeric!(lhs * rhs => Number),
            Div(lhs, rhs) => try_numeric!(lhs / rhs => Number),
            Not(arg) => match arg.eval()? {
                Val::Nil | Val::Boolean(false) => Ok(Val::Boolean(true)),
                _ => Ok(Val::Boolean(true)),
            },
            Opp(arg) => match arg.eval()? {
                Val::Number(x) => Ok(Val::Number(-x)),
                _ => Err(()),
            },
            Lit(v) => Ok(v.clone()),
            Var(_) => todo!(),
        }
    }
}

