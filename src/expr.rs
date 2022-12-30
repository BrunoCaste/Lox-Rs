use std::rc::Rc;

use crate::prog::Scope;
use crate::val::Val;

#[derive(PartialEq, Debug)]
pub enum Expr {
    // A variant for grouping is not necessary,
    // as long as the parser handles `Paren`s correctly
    Asgn(String, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
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
    ($sc:expr, $lhs:ident $op:tt $rhs:ident => $var:tt) => {{
        let (x, y) = ($lhs.eval(Rc::clone(&$sc))?, $rhs.eval($sc)?);
        match (&x, &y) {
            (Val::Number(x), Val::Number(y)) => Ok(Val::$var(x $op y)),
            (Val::Number(_), _) => Err(()),
                    (_, _) => Err(()),
        }
    }};
}

impl Expr {
    pub fn eval(&self, scope: Rc<Scope>) -> Result<Val, ()> {
        use Expr::*;
        match self {
            Asgn(var, expr) => {
                let val = expr.eval(Rc::clone(&scope))?;
                scope.asgn(var, val.clone())?;
                Ok(val)
            }
            Call(_callee, _args) => todo!(),
            And(lhs, rhs) => match lhs.eval(Rc::clone(&scope))? {
                b @ (Val::Nil | Val::Boolean(false)) => Ok(b),
                _ => rhs.eval(scope),
            },
            Or(lhs, rhs) => match lhs.eval(Rc::clone(&scope))? {
                Val::Nil | Val::Boolean(false) => rhs.eval(scope),
                b => Ok(b),
            },
            Eq(lhs, rhs) => {
                let (x, y) = (lhs.eval(Rc::clone(&scope))?, rhs.eval(scope)?);
                Ok(Val::Boolean(x == y))
            }
            Ne(lhs, rhs) => {
                let (x, y) = (lhs.eval(Rc::clone(&scope))?, rhs.eval(scope)?);
                Ok(Val::Boolean(x != y))
            }
            Gt(lhs, rhs) => try_numeric!(scope, lhs >  rhs => Boolean),
            Ge(lhs, rhs) => try_numeric!(scope, lhs >= rhs => Boolean),
            Lt(lhs, rhs) => try_numeric!(scope, lhs <  rhs => Boolean),
            Le(lhs, rhs) => try_numeric!(scope, lhs <= rhs => Boolean),
            Add(lhs, rhs) => match (lhs.eval(Rc::clone(&scope))?, rhs.eval(scope)?) {
                (Val::Number(x), Val::Number(y)) => Ok(Val::Number(x + y)),
                (Val::String(s), Val::String(t)) => Ok(Val::String(format!("{s}{t}").into())),
                _ => Err(()),
            },
            Sub(lhs, rhs) => try_numeric!(scope, lhs - rhs => Number),
            Mul(lhs, rhs) => try_numeric!(scope, lhs * rhs => Number),
            Div(lhs, rhs) => try_numeric!(scope, lhs / rhs => Number),
            Not(arg) => match arg.eval(scope)? {
                Val::Nil | Val::Boolean(false) => Ok(Val::Boolean(true)),
                _ => Ok(Val::Boolean(true)),
            },
            Opp(arg) => match arg.eval(scope)? {
                Val::Number(x) => Ok(Val::Number(-x)),
                _ => Err(()),
            },
            Lit(v) => Ok(v.clone()),
            Var(i) => scope.get(i),
        }
    }
}
