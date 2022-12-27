use std::iter::Peekable;

use crate::{
    expr::{Expr, Val},
    lexer::TokKind,
    prog::Prog,
    stmt::Stmt,
};

pub use crate::lexer::Token;
pub trait Parser<Output> {
    type Error;

    fn parse(lexer: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Output, Self::Error>;
}

type CompilationError = ();

pub struct RecursiveDescent<T>(std::marker::PhantomData<T>);

impl Parser<Prog> for RecursiveDescent<Prog> {
    type Error = ();

    fn parse(lexer: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Prog, Self::Error> {
        let mut program = Prog(Vec::new());

        while lexer.peek().is_some() {
            program.0.push(RecursiveDescent::<Stmt>::parse(lexer)?);
        }
        Ok(program)
    }
}

impl Parser<Stmt> for RecursiveDescent<Stmt> {
    type Error = ();

    fn parse(lexer: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Stmt, Self::Error> {
        if lexer.next_if(|t| t.kind == TokKind::Let).is_some() {
            Self::parse_decl(lexer)
        } else {
            Self::parse_stmt(lexer)
        }
        .map_err(|e| {
            // Self::sync(lexer);
            e
        })
    }
}

impl RecursiveDescent<Stmt> {
    fn parse_stmt(
        lexer: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> Result<Stmt, <Self as Parser<Stmt>>::Error> {
        let stmt = if let Some(tok) =
            lexer.next_if(|t| matches!(t.kind, TokKind::LBrace | TokKind::Print))
        {
            match tok.kind {
                TokKind::LBrace => {
                    let mut block = Vec::new();
                    while lexer.peek().is_some_and(|t| t.kind != TokKind::RBrace) {
                        block.push(RecursiveDescent::parse(lexer)?)
                    }
                    Stmt::Block(block)
                }
                TokKind::Print => Stmt::Print(RecursiveDescent::parse(lexer)?),
                _ => unreachable!(),
            }
        } else {
            Stmt::Expr(RecursiveDescent::parse(lexer)?)
        };

        match stmt {
            Stmt::Expr(_) | Stmt::Decl(_, _) | Stmt::Print(_) => {
                if lexer.next_if(|t| t.kind == TokKind::Semicolon).is_none() {
                    println!("Expected ; after statement");
                    return Err(());
                }
            }
            Stmt::Block(_) => {
                if lexer.next_if(|t| t.kind == TokKind::RBrace).is_none() {
                    println!("Expected }} after block");
                    return Err(());
                }
            }
        }

        Ok(stmt)
    }

    fn parse_decl(
        lexer: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> Result<Stmt, <Self as Parser<Stmt>>::Error> {
        let var = match lexer.next() {
            Some(Token {
                kind: TokKind::Ident(i),
                loc: _,
            }) => i,
            Some(Token { kind, loc: _ }) => {
                println!("Expected identifier, found {kind:?}");
                return Err(());
            }
            None => {
                println!("EOF error");
                return Err(());
            }
        };

        let init = if lexer.next_if(|t| t.kind == TokKind::Equal).is_some() {
            Some(RecursiveDescent::<Expr>::parse(lexer)?)
        } else {
            None
        };

        if lexer.next_if(|t| t.kind == TokKind::Semicolon).is_none() {
            println!("Expected ; after declaration");
            return Err(());
        }

        Ok(Stmt::Decl(var, init))
    }
}

impl Parser<Expr> for RecursiveDescent<Expr> {
    type Error = ();

    fn parse(lexer: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Expr, Self::Error> {
        Self::parse_asgn(lexer)
    }
}

impl RecursiveDescent<Expr> {
    fn parse_asgn(
        lexer: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> Result<Expr, CompilationError> {
        let target = Self::parse_log(lexer)?;

        if lexer.next_if(|t| t.kind == TokKind::Equal).is_some() {
            if let Expr::Var(i) = target {
                let value = Self::parse_asgn(lexer)?;
                Ok(Expr::Asgn(i, Box::new(value)))
            } else {
                println!("Invalid asignment target");
                Err(())
            }
        } else {
            Ok(target)
        }
    }

    fn parse_log(
        lexer: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> Result<Expr, CompilationError> {
        use crate::lexer::TokKind::*;

        let mut lhs = Self::parse_cmp(lexer)?;

        while let Some(op) = lexer.next_if(|t| matches!(t.kind, And | Or)) {
            let rhs = Self::parse_cmp(lexer)?;

            lhs = match op.kind {
                And => Expr::And(Box::new(lhs), Box::new(rhs)),
                Or => Expr::Or(Box::new(lhs), Box::new(rhs)),
                _ => unreachable!(),
            }
        }
        Ok(lhs)
    }

    fn parse_cmp(
        lexer: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> Result<Expr, CompilationError> {
        use crate::lexer::TokKind::*;

        let mut lhs = Self::parse_term(lexer)?;

        while let Some(op) = lexer.next_if(|t| {
            matches!(
                t.kind,
                BangEqual | EqualEqual | Less | Greater | LessEqual | GreaterEqual
            )
        }) {
            let rhs = Self::parse_term(lexer)?;

            lhs = match op.kind {
                BangEqual => Expr::Ne(Box::new(lhs), Box::new(rhs)),
                EqualEqual => Expr::Eq(Box::new(lhs), Box::new(rhs)),
                Less => Expr::Lt(Box::new(lhs), Box::new(rhs)),
                Greater => Expr::Gt(Box::new(lhs), Box::new(rhs)),
                LessEqual => Expr::Le(Box::new(lhs), Box::new(rhs)),
                GreaterEqual => Expr::Ge(Box::new(lhs), Box::new(rhs)),
                _ => unreachable!(),
            }
        }
        Ok(lhs)
    }

    fn parse_term(
        lexer: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> Result<Expr, CompilationError> {
        use crate::lexer::TokKind::*;

        let mut lhs = Self::parse_factor(lexer)?;

        while let Some(op) = lexer.next_if(|t| matches!(t.kind, Plus | Minus)) {
            let rhs = Self::parse_factor(lexer)?;

            lhs = match op.kind {
                Plus => Expr::Add(Box::new(lhs), Box::new(rhs)),
                Minus => Expr::Sub(Box::new(lhs), Box::new(rhs)),
                _ => unreachable!(),
            }
        }
        Ok(lhs)
    }

    fn parse_factor(
        lexer: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> Result<Expr, CompilationError> {
        use crate::lexer::TokKind::*;

        let mut lhs = Self::parse_unary(lexer)?;

        while let Some(op) = lexer.next_if(|t| matches!(t.kind, Star | Slash)) {
            let rhs = Self::parse_unary(lexer)?;

            lhs = match op.kind {
                Star => Expr::Mul(Box::new(lhs), Box::new(rhs)),
                Slash => Expr::Div(Box::new(lhs), Box::new(rhs)),
                _ => unreachable!(),
            }
        }
        Ok(lhs)
    }

    fn parse_unary(
        lexer: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> Result<Expr, CompilationError> {
        use crate::lexer::TokKind::*;

        if let Some(op) = lexer.next_if(|t| matches!(t.kind, Bang | Minus)) {
            let arg = Self::parse_unary(lexer)?;

            Ok(match op.kind {
                Bang => Expr::Not(Box::new(arg)),
                Minus => Expr::Opp(Box::new(arg)),
                _ => unreachable!(),
            })
        } else {
            Self::parse_primary(lexer)
        }
    }

    fn parse_primary(
        lexer: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> Result<Expr, CompilationError> {
        use crate::lexer::TokKind::*;

        match lexer.next() {
            None => {
                println!("EOF error");
                Err(())
            }
            Some(t) => match t.kind {
                Nil => Ok(Expr::Lit(Val::Nil)),
                True => Ok(Expr::Lit(Val::Boolean(true))),
                False => Ok(Expr::Lit(Val::Boolean(false))),
                Number(x) => Ok(Expr::Lit(Val::Number(x))),
                String(s) => Ok(Expr::Lit(Val::String(s))),
                Ident(s) => Ok(Expr::Var(s)),
                LParen => {
                    let inner = Self::parse_log(lexer)?;
                    let closing = lexer.next();
                    if closing.is_some_and(|t| t.kind == RParen) {
                        Ok(inner)
                    } else {
                        println!("Unclosed paren");
                        Err(())
                    }
                }
                x => {
                    println!("Unexpected Token: {x:?}");
                    Err(())
                }
            },
        }
    }
}

#[cfg(test)]
mod test {
    use crate::lexer::Lexer;

    use super::*;

    #[test]
    fn trailing_chars() {
        let mut l = Lexer::new("6 + hello + 8 ;".chars()).peekable();
        let _: Expr = RecursiveDescent::parse(&mut l).unwrap();

        assert_ne!(l.next(), None);
    }

    #[test]
    fn left_asoc() {
        use Expr::*;

        let l = Lexer::new("6 + 3 + 8".chars());
        let e = RecursiveDescent::parse(&mut l.peekable());

        assert_eq!(
            e,
            Ok(Add(
                Box::new(Add(
                    Box::new(Lit(Val::Number(6.0))),
                    Box::new(Lit(Val::Number(3.0))),
                )),
                Box::new(Lit(Val::Number(8.0))),
            ))
        );
    }

    #[test]
    fn right_asoc() {
        use Expr::*;

        let l = Lexer::new("a = b = 3".chars());
        let e = RecursiveDescent::parse(&mut l.peekable());

        assert_eq!(
            e,
            Ok(Asgn(
                "a".to_string(),
                Box::new(Asgn("b".to_string(), Box::new(Lit(Val::Number(3.0))),)),
            ))
        );
    }

    #[test]
    fn grouping() {
        use Expr::*;

        let l = Lexer::new("x + (3 + 8)".chars());
        let e = RecursiveDescent::parse(&mut l.peekable());

        assert_eq!(
            e,
            Ok(Add(
                Box::new(Var("x".to_string())),
                Box::new(Add(
                    Box::new(Lit(Val::Number(3.0))),
                    Box::new(Lit(Val::Number(8.0))),
                )),
            ))
        );
    }

    #[test]
    fn prec_increasing() {
        use Expr::*;

        let l = Lexer::new("x = true and 0 != 2 + 6 / -!false".chars());
        let e = RecursiveDescent::parse(&mut l.peekable());

        assert_eq!(
            e,
            Ok(Asgn(
                "x".to_string(),
                Box::new(And(
                    Box::new(Lit(Val::Boolean(true))),
                    Box::new(Ne(
                        Box::new(Lit(Val::Number(0.0))),
                        Box::new(Add(
                            Box::new(Lit(Val::Number(2.0))),
                            Box::new(Div(
                                Box::new(Lit(Val::Number(6.0))),
                                Box::new(Opp(Box::new(Not(Box::new(Lit(Val::Boolean(false))))))),
                            )),
                        )),
                    )),
                ))
            ))
        );
    }

    #[test]
    fn prec_decreasing() {
        use Expr::*;

        let l = Lexer::new("-!false / 6 + 2 != 0 and true".chars());
        let e = RecursiveDescent::parse(&mut l.peekable());

        assert_eq!(
            e,
            Ok(And(
                Box::new(Ne(
                    Box::new(Add(
                        Box::new(Div(
                            Box::new(Opp(Box::new(Not(Box::new(Lit(Val::Boolean(false))))))),
                            Box::new(Lit(Val::Number(6.0))),
                        )),
                        Box::new(Lit(Val::Number(2.0))),
                    )),
                    Box::new(Lit(Val::Number(0.0))),
                )),
                Box::new(Lit(Val::Boolean(true))),
            ))
        );
    }

    #[test]
    fn eof_error() {
        let l = Lexer::new("2 + - 6 / ".chars());
        let e = RecursiveDescent::<Expr>::parse(&mut l.peekable());

        assert!(e.is_err());
    }

    #[test]
    fn unclosed_paren() {
        let l = Lexer::new("2 + - (6 / 4".chars());
        let e = RecursiveDescent::<Expr>::parse(&mut l.peekable());
        assert!(e.is_err());
    }

    #[test]
    fn asgn_target_error() {
        let l = Lexer::new("6 = 3 + 8".chars());
        let e = RecursiveDescent::<Expr>::parse(&mut l.peekable());
        assert!(e.is_err());
    }
}
