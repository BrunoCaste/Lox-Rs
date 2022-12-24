use std::iter::Peekable;

use crate::{
    expr::{Expr, Val},
    lexer::TokKind,
};

pub use crate::lexer::Token;
pub trait Parser<Output> {
    type Error;

    fn parse(lexer: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Output, Self::Error>;
}

type CompilationError = ();

pub struct RecursiveDescent<T>(std::marker::PhantomData<T>);

impl Parser<Expr> for RecursiveDescent<Expr> {
    type Error = ();

    fn parse(lexer: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Expr, Self::Error> {
        Self::parse_log(&mut lexer.peekable())
    }
}

impl RecursiveDescent<Expr> {
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
            None => Err(()), /* EOF error */
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
                        Err(()) // Unclosed Paren
                    }
                }
                _ => Err(()), // Unexpected Token
            },
        }
    }
}

#[cfg(test)]
mod test {
    use crate::lexer::Lexer;

    use super::*;

    #[test]
    fn asoc() {
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

        let l = Lexer::new("true and 0 != 2 + 6 / -!false".chars());
        let e = RecursiveDescent::parse(&mut l.peekable());

        assert_eq!(
            e,
            Ok(And(
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

        assert_eq!(e, Err(()));
    }

    #[test]
    fn unclosed_paren() {
        let l = Lexer::new("2 + - (6 / 4".chars());
        let e = RecursiveDescent::<Expr>::parse(&mut l.peekable());
        assert_eq!(e, Err(()));
    }
}
