#![feature(is_some_and)]

use std::iter::Peekable;

#[derive(PartialEq, Debug)]
pub enum Token {
    // Keywords
    And,
    Class,
    Else,
    False,
    Fn,
    For,
    If,
    Let,
    Nil,
    Or,
    Print,
    Return,
    This,
    True,
    While,
    // Single-character symbols
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Star,

    Bang,
    Equal,
    Less,
    Greater,
    Slash,
    // Two-character symbols
    BangEqual,
    EqualEqual,
    LessEqual,
    GreaterEqual,
    // Literals
    String(String),
    Number(f64),

    Ident(String),

    Comment,
    Invalid,
    Unterminated,
}

struct Lexer<I: Iterator<Item = char>> {
    src: Peekable<I>,
}

impl<I> Lexer<I>
where
    I: Iterator<Item = char>,
{
    pub fn new(src: I) -> Self {
        Self {
            src: src.peekable(),
        }
    }

    fn eat_while(&mut self, mut f: impl FnMut(char) -> bool) {
        while self.src.next_if(|&c| f(c)).is_some() { /* spin */ }
    }

    fn next_raw(&mut self) -> Option<Token> {
        self.eat_while(char::is_whitespace);

        use Token::*;
        self.src.next().map(|c| match c {
            '(' => LParen,
            ')' => RParen,
            '{' => LBrace,
            '}' => RBrace,
            ',' => Comma,
            '.' => Dot,
            '-' => Minus,
            '+' => Plus,
            ';' => Semicolon,
            '*' => Star,
            '!' => {
                if self.src.next_if(|c| c == &'=').is_some() {
                    BangEqual
                } else {
                    Bang
                }
            }
            '=' => {
                if self.src.next_if(|c| c == &'=').is_some() {
                    EqualEqual
                } else {
                    Equal
                }
            }
            '<' => {
                if self.src.next_if(|c| c == &'=').is_some() {
                    LessEqual
                } else {
                    Less
                }
            }
            '>' => {
                if self.src.next_if(|c| c == &'=').is_some() {
                    GreaterEqual
                } else {
                    Greater
                }
            }
            '/' => {
                if self.src.next_if(|c| c == &'/').is_some() {
                    self.eat_while(|c| c != '\n');
                    Comment
                } else {
                    Slash
                }
            }
            _ => todo!(),
        })
    }
}

impl<I> Iterator for Lexer<I>
where
    I: Iterator<Item = char>,
{
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(t) = self.next_raw() {
            if t != Token::Comment {
                return Some(t);
            }
        }
        None
    }
}

fn main() {
    println!("Hello, world!");
}
