use crate::{
    expr::{Expr, Val},
    prog::Scope,
};

#[derive(PartialEq, Debug)]
pub enum Stmt {
    Expr(Expr),
    Print(Expr),
    Decl(String, Option<Expr>),
}

impl Stmt {
    pub fn exec(&self, scope: &mut Scope) -> Result<(), ()> {
        match self {
            Self::Expr(e) => e.eval(scope).map(|_| ()),
            Self::Print(e) => {
                let e = e.eval(scope)?;
                println!("{e}");
                Ok(())
            }
            Self::Decl(name, expr) => {
                let init = if let Some(e) = expr {
                    e.eval(scope)?
                } else {
                    Val::Nil
                };
                scope.def(name, init);
                Ok(())
            }
        }
    }
}
