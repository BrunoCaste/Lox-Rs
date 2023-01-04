use crate::lexer::{Loc, TokKind, Token};

#[derive(PartialEq, Debug)]
pub enum ParserError {
    Expected { exp: TokKind, fnd: Option<Token> },
    TooManyParams { loc: Loc },
    TooManyArgs { loc: Loc },
    InvalidAsgn { loc: Loc },
    Unmatched { open: Token, hint: Option<Loc> },
    Unexpected { tok: Token },
    EOF,
}
