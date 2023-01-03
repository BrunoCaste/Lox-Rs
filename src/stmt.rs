use std::rc::Rc;

use crate::{expr::Expr, prog::Scope, val::Val};

#[derive(PartialEq, Debug, Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expr(Expr),
    Print(Expr),
    Decl(String, Option<Expr>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
}

impl Stmt {
    pub fn exec(&self, scope: Rc<Scope>) -> Result<Val, ()> {
        match self {
            Self::Block(stmts) => {
                // let inner = Scope::from(scope);
                let inner = Scope::inner(&scope);
                for s in stmts {
                    let val = s.exec(Rc::clone(&inner))?;
                    if val != Val::NoVal {
                        return Ok(val);
                    }
                }
                Ok(Val::NoVal)
            }
            Self::Expr(e) => e.eval(scope).map(|_| Val::NoVal),
            Self::Print(e) => {
                let e = e.eval(scope)?;
                println!("{e}");
                Ok(Val::NoVal)
            }
            Self::Decl(name, expr) => {
                let init = if let Some(e) = expr {
                    e.eval(Rc::clone(&scope))?
                } else {
                    Val::Nil
                };
                scope.def(name, init);
                Ok(Val::NoVal)
            }
            Self::If(cond, then_branch, else_branch) => {
                let ret = if cond.eval(Rc::clone(&scope))?.into() {
                    then_branch.exec(scope)?
                } else if let Some(else_branch) = else_branch {
                    else_branch.exec(scope)?
                } else {
                    Val::NoVal
                };
                Ok(ret)
            }
            Self::While(cond, body) => {
                let mut ret = Val::NoVal;
                while cond.eval(Rc::clone(&scope))?.into() {
                    ret = body.exec(Rc::clone(&scope))?;
                    if ret != Val::NoVal {
                        break;
                    }
                }
                Ok(ret)
            }
            }
        }
    }
}
