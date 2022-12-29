use std::iter::Peekable;

use crate::{expr::Expr, lexer::TokKind::*, prog::Prog, stmt::Stmt, val::Val};

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
        if lexer.next_if(|t| t.kind == Let).is_some() {
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
            lexer.next_if(|t| matches!(t.kind, LBrace | Print | If | While | For))
        {
            match tok.kind {
                LBrace => {
                    let block = Self::parse_block(lexer)?;
                    if lexer.next_if(|t| t.kind == RBrace).is_none() {
                        println!("Expected }} after block");
                        return Err(());
                    };
                    block
                }
                Print => Stmt::Print(RecursiveDescent::parse(lexer)?),
                If => {
                    if lexer.next_if(|t| t.kind == LParen).is_none() {
                        println!("Expected '(' after 'if'");
                        return Err(());
                    }
                    let cond = RecursiveDescent::parse(lexer)?;
                    if lexer.next_if(|t| t.kind == RParen).is_none() {
                        println!("Expected ')' after condition");
                        return Err(());
                    }
                    let body = Self::parse_stmt(lexer)?;
                    let otherwise = if lexer.next_if(|t| t.kind == Else).is_some() {
                        Some(Self::parse_stmt(lexer)?)
                    } else {
                        None
                    };
                    Stmt::If(cond, Box::new(body), otherwise.map(Box::new))
                }
                While => {
                    if lexer.next_if(|t| t.kind == LParen).is_none() {
                        println!("Expected '(' after 'while'");
                        return Err(());
                    }
                    let cond = RecursiveDescent::parse(lexer)?;
                    if lexer.next_if(|t| t.kind == RParen).is_none() {
                        println!("Expected ')' after condition");
                        return Err(());
                    }
                    let body = Self::parse_stmt(lexer)?;
                    Stmt::While(cond, Box::new(body))
                }
                For => Self::parse_for(lexer)?,
                _ => unreachable!(),
            }
        } else {
            Stmt::Expr(RecursiveDescent::parse(lexer)?)
        };

        match stmt {
            Stmt::Expr(_) | Stmt::Decl(_, _) | Stmt::Print(_)
                if lexer.next_if(|t| t.kind == Semicolon).is_none() =>
            {
                println!("Expected ; after statement");
                return Err(());
            }
            _ => {}
        }

        Ok(stmt)
    }

    fn parse_decl(
        lexer: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> Result<Stmt, <Self as Parser<Stmt>>::Error> {
        let var = match lexer.next() {
            Some(Token {
                kind: Ident(i),
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

        let init = if lexer.next_if(|t| t.kind == Equal).is_some() {
            Some(RecursiveDescent::<Expr>::parse(lexer)?)
        } else {
            None
        };

        if lexer.next_if(|t| t.kind == Semicolon).is_none() {
            println!("Expected ; after declaration");
            return Err(());
        }

        Ok(Stmt::Decl(var, init))
    }

    fn parse_block(
        lexer: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> Result<Stmt, <Self as Parser<Stmt>>::Error> {
        let mut block = Vec::new();
        while lexer.peek().is_some_and(|t| t.kind != RBrace) {
            block.push(RecursiveDescent::parse(lexer)?);
        }
        Ok(Stmt::Block(block))
    }

    fn parse_for(
        lexer: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> Result<Stmt, <Self as Parser<Stmt>>::Error> {
        if lexer.next_if(|t| t.kind == LParen).is_none() {
            println!("Expected '(' after 'for'");
            return Err(());
        }
        // parse init
        let init = if let Some(tok) = lexer
            .next_if(|t| matches!(t.kind, Let | Semicolon))
            .map(|t| t.kind)
        {
            match tok {
                Semicolon => None,
                Let => Some(Self::parse_decl(lexer)?),
                _ => unreachable!(),
            }
        } else {
            let expr = RecursiveDescent::parse(lexer)?;
            if lexer.next_if(|t| t.kind == Semicolon).is_none() {
                println!("Expected ; after expression");
                return Err(());
            } else {
                Some(Stmt::Expr(expr))
            }
        };
        // parse cond
        let cond = if lexer.peek().is_some_and(|t| t.kind == Semicolon) {
            Expr::Lit(Val::Boolean(true))
        } else {
            RecursiveDescent::parse(lexer)?
        };
        if lexer.next_if(|t| t.kind == Semicolon).is_none() {
            println!("Expected ; after loop condition");
            return Err(());
        };
        // parse increment
        let increment = if lexer.peek().is_some_and(|t| t.kind == RParen) {
            None
        } else {
            Some(RecursiveDescent::parse(lexer)?)
        };
        if lexer.next_if(|t| t.kind == RParen).is_none() {
            println!("Expected ')' after for clauses");
            return Err(());
        };
        // parse body
        let body = Self::parse_stmt(lexer)?;
        // assemble loop
        let body = if let Some(inc) = increment {
            match body {
                Stmt::Block(mut vec) => {
                    vec.push(Stmt::Expr(inc));
                    Stmt::Block(vec)
                }
                stmt => Stmt::Block(vec![stmt, Stmt::Expr(inc)]),
            }
        } else {
            body
        };
        let desugared_loop = Stmt::While(cond, Box::new(body));
        Ok(if let Some(init) = init {
            Stmt::Block(vec![init, desugared_loop])
        } else {
            desugared_loop
        })
    }
}

/*
* expr    -> asgn
* asgn    -> IDENT "=" asgn | logic
* logic   -> cmp | logic ("and" | "or") cmp
* cmp     -> term | cmp ("==" | "!=" | "<" | "<=" | ">" | ">=") term
* term    -> factor | term ("+" | "-") factor
* factor  -> unary | factor ("*" | "/") unary
* unary   -> ("!" | "-") unary | call
* call    -> (call | primary) "(" args ")"
* primary -> TRUE | FALSE | NIL | NUMBER | STRING | IDENT | "(" expr ")"
*
* args -> expr ("," expr)* | EPSILON
*/

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

        if lexer.next_if(|t| t.kind == Equal).is_some() {
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
        if let Some(op) = lexer.next_if(|t| matches!(t.kind, Bang | Minus)) {
            let arg = Self::parse_unary(lexer)?;

            Ok(match op.kind {
                Bang => Expr::Not(Box::new(arg)),
                Minus => Expr::Opp(Box::new(arg)),
                _ => unreachable!(),
            })
        } else {
            Self::parse_call(lexer)
        }
    }

    fn parse_call(
        lexer: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> Result<Expr, CompilationError> {
        let mut callee = Self::parse_primary(lexer)?;
        while lexer.next_if(|t| matches!(t.kind, LParen)).is_some() {
            let args = Self::parse_args(lexer)?;
            callee = Expr::Call(Box::new(callee), args);
            if lexer.next_if(|t| matches!(t.kind, RParen)).is_none() {
                println!("Unmatched parenthesis");
                return Err(());
            }
        }
        Ok(callee)
    }

    fn parse_args(
        lexer: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> Result<Vec<Expr>, CompilationError> {
        let mut args = Vec::new();
        if lexer.peek().is_some_and(|t| t.kind != RParen) {
            args.push(Self::parse(lexer)?);
            while lexer.next_if(|t| t.kind == Comma).is_some() {
                if args.len() > 255 {
                    println!("argument count (255) exceeded");
                    return Err(());
                }
                args.push(Self::parse(lexer)?);
            }
        }
        Ok(args)
    }

    fn parse_primary(
        lexer: &mut Peekable<impl Iterator<Item = Token>>,
    ) -> Result<Expr, CompilationError> {
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
