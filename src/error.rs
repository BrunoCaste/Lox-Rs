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

fn message_at_location(src: &str, loc: &Loc, msg: &str) {
    eprintln!(
        "{:>4} | {}",
        loc.row,
        src.lines()
            .nth(loc.row)
            .expect("Errors should be reported on an existing line")
    );
    eprintln!("       {}{msg}", " ".repeat(loc.col))
}

pub trait Report {
    fn report(&self, code: &str);
}

impl Report for ParserError {
    fn report(&self, code: &str) {
        use ParserError::*;
        eprint!("error[lox]: ");

        match self {
            Expected { exp, fnd: None } => {
                eprintln!("expected {exp}, found end of file");
                let (lineno, line) = code.lines().enumerate().last().unwrap();
                eprintln!("{lineno:>4} | {line}");
                eprintln!("       {}^ EOF found here", " ".repeat(line.len()));
            }
            Expected {
                exp,
                fnd: Some(tok),
            } => {
                eprintln!("syntax error: expected {exp}, found {}", tok.kind);
                message_at_location(code, &tok.loc, "^ here");
            }
            TooManyParams { loc } => {
                message_at_location(code, loc, "^ this is the 256th parameter");
            }
            TooManyArgs { loc } => {
                eprintln!("functions cannot take more than 255 arguments");
                message_at_location(code, loc, "^ this is the 256th argument");
            }
            InvalidAsgn { loc } => {
                eprintln!("invalid assignment target");
                message_at_location(code, loc, "^ only variables may be assigned a value");
            }
            Unmatched { open, hint } => {
                eprintln!("unmatched {}", open.kind);
                if let Some(hint) = hint {
                    message_at_location(code, &open.loc, "^ unclosed delimiter here...");
                    eprintln!("...");
                    message_at_location(code, hint, "^ ... may have closing delimiter here");
                } else {
                    message_at_location(code, &open.loc, "^ unclosed delimiter here");
                }
            }
            Unexpected { tok } => {
                eprintln!("unexpected token: {}", tok.kind);
                message_at_location(code, &tok.loc, "^");
            }
            EOF => {
                eprintln!("unexpected end of file");
                let (lineno, line) = code.lines().enumerate().last().unwrap();
                eprintln!("{lineno:>4} | {line}");
                eprintln!("       {}^ EOF found here", " ".repeat(line.len()));
            }
        }
    }
}
