#[derive(PartialEq, Debug)]
pub enum Val {
    Number(f64),
    Boolean(bool),
    String(String),
    Nil,
}

#[derive(PartialEq, Debug)]
pub enum Expr {
    // A variant for grouping is not necessary,
    // as long as the parser handles `Paren`s correctly
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
}

pub use crate::lexer::Token;
pub trait Parser {
    type Output;
    type Error;

    fn parse(lexer: &mut impl Iterator<Item = Token>) -> Result<Self::Output, Self::Error>;
}
