use std::iter::Peekable;

use crate::{
    error::ParserError,
    lexer::{
        Loc,
        TokKind::{self, *},
        Token,
    },
};

mod rec_desc;
pub use rec_desc::RecursiveDescent;

pub trait Parser<Output> {
    fn parse(lexer: &mut Peekable<impl Iterator<Item = Token>>) -> Result<Output, ParserError>;
}

fn consume(
    lexer: &mut Peekable<impl Iterator<Item = Token>>,
    expected: TokKind,
) -> Result<Token, ParserError> {
    if let Some(t) = lexer.next_if(|t| t.kind == expected) {
        Ok(t)
    } else {
        Err(ParserError::Expected {
            exp: expected,
            fnd: lexer.peek().cloned(),
        })
    }
}

fn consume_ident(
    lexer: &mut Peekable<impl Iterator<Item = Token>>,
) -> Result<(String, Loc), ParserError> {
    if let Some(Token {
        kind: Ident(name),
        loc,
    }) = lexer.next_if(|t| matches!(t.kind, Ident(_)))
    {
        Ok((name, loc))
    } else {
        Err(ParserError::Expected {
            exp: Ident(Default::default()),
            fnd: lexer.peek().cloned(),
        })
    }
}
