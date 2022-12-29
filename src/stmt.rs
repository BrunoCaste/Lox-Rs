use crate::{expr::Expr, prog::Scope, val::Val};

#[derive(PartialEq, Debug)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expr(Expr),
    Print(Expr),
    Decl(String, Option<Expr>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
}

impl Stmt {
    pub fn exec(&self, scope: &mut Scope) -> Result<Val, ()> {
        match self {
            Self::Block(stmts) => {
                // let inner = Scope::from(scope);
                scope.add_inner();
                for s in stmts {
                    s.exec(scope).map_err(|e| {
                        scope.exit_inner();
                        e
                    })?;
                }
                scope.exit_inner();
                Ok(Val::Nil)
            }
            Self::Expr(e) => e.eval(scope).map(|_| Val::Nil),
            Self::Print(e) => {
                let e = e.eval(scope)?;
                println!("{e}");
                Ok(Val::Nil)
            }
            Self::Decl(name, expr) => {
                let init = if let Some(e) = expr {
                    e.eval(scope)?
                } else {
                    Val::Nil
                };
                scope.def(name, init);
                Ok(Val::Nil)
            }
            Self::If(cond, then_branch, else_branch) => {
                if cond.eval(scope)?.into() {
                    then_branch.exec(scope)?;
                } else if let Some(else_branch) = else_branch {
                    else_branch.exec(scope)?;
                }
                Ok(Val::Nil)
            }
            Self::While(cond, body) => {
                while cond.eval(scope)?.into() {
                    body.exec(scope)?;
                }
                Ok(Val::Nil)
            }
        }
    }
}
