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
    // Used to construct literals and identifiers
    // and to avoid repeated allocations
    buf: String,
}

impl<I> Lexer<I>
where
    I: Iterator<Item = char>,
    I: Clone,
{
    const BUF_CAP: usize = 64;

    pub fn new(src: I) -> Self {
        Self {
            src: src.peekable(),
            buf: String::with_capacity(Self::BUF_CAP),
        }
    }

    fn eat_while(&mut self, mut f: impl FnMut(char) -> bool) {
        while self.src.next_if(|&c| f(c)).is_some() { /* spin */ }
    }

    fn buf_while(&mut self, mut f: impl FnMut(char) -> bool) {
        while let Some(c) = self.src.next_if(|&c| f(c)) {
            self.buf.push(c);
        }
    }

    fn peek_snd(&self) -> Option<char> {
        let mut other = self.src.clone();
        other.next();
        other.next()
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
            x if x.is_ascii_alphabetic() || x == '_' => {
                self.buf.clear();
                self.buf.push(x);
                self.buf_while(|c| c.is_ascii_alphanumeric() || c == '_');
                Ident(self.buf.to_string())
            }
            x if x.is_ascii_digit() => {
                self.buf.clear();
                self.buf.push(x);
                self.buf_while(|c| c.is_ascii_digit());
                if self.src.peek().is_some_and(|&c| c == '.')
                    && (self.peek_snd().is_some_and(|c| c.is_ascii_digit()))
                {
                    self.src.next();
                    self.buf.push('.');
                    self.buf_while(|c| c.is_ascii_digit())
                }
                Token::Number(self.buf.parse().unwrap())
            }
            _ => todo!(),
        })
    }
}

impl<I> Iterator for Lexer<I>
where
    I: Iterator<Item = char>,
    I: Clone,
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

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_lexer_punctuation() {
        let mut l = Lexer::new("(){};,+-*!===<=>=!=<>/.\n".chars());
        assert_eq!(l.next(), Some(Token::LParen));
        assert_eq!(l.next(), Some(Token::RParen));
        assert_eq!(l.next(), Some(Token::LBrace));
        assert_eq!(l.next(), Some(Token::RBrace));
        assert_eq!(l.next(), Some(Token::Semicolon));
        assert_eq!(l.next(), Some(Token::Comma));
        assert_eq!(l.next(), Some(Token::Plus));
        assert_eq!(l.next(), Some(Token::Minus));
        assert_eq!(l.next(), Some(Token::Star));
        assert_eq!(l.next(), Some(Token::BangEqual));
        assert_eq!(l.next(), Some(Token::EqualEqual));
        assert_eq!(l.next(), Some(Token::LessEqual));
        assert_eq!(l.next(), Some(Token::GreaterEqual));
        assert_eq!(l.next(), Some(Token::BangEqual));
        assert_eq!(l.next(), Some(Token::Less));
        assert_eq!(l.next(), Some(Token::Greater));
        assert_eq!(l.next(), Some(Token::Slash));
        assert_eq!(l.next(), Some(Token::Dot));
        assert_eq!(l.next(), None);
    }

    #[test]
    fn test_lexer_identifiers() {
        let mut l = Lexer::new(
            "andy formless fo _ _123 _abc ab123
abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890_"
                .chars(),
        );
        assert_eq!(l.next(), Some(Token::Ident("andy".to_string())));
        assert_eq!(l.next(), Some(Token::Ident("formless".to_string())));
        assert_eq!(l.next(), Some(Token::Ident("fo".to_string())));
        assert_eq!(l.next(), Some(Token::Ident("_".to_string())));
        assert_eq!(l.next(), Some(Token::Ident("_123".to_string())));
        assert_eq!(l.next(), Some(Token::Ident("_abc".to_string())));
        assert_eq!(l.next(), Some(Token::Ident("ab123".to_string())));
        assert_eq!(
            l.next(),
            Some(Token::Ident(
                "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890_".to_string()
            ))
        );
        assert_eq!(l.next(), None);
    }

    #[test]
    fn test_lexer_numbers() {
        let mut l = Lexer::new("123 123.456 .456 123.".chars());
        assert_eq!(l.next(), Some(Token::Number(123.0)));
        assert_eq!(l.next(), Some(Token::Number(123.456)));
        assert_eq!(l.next(), Some(Token::Dot));
        assert_eq!(l.next(), Some(Token::Number(456.0)));
        assert_eq!(l.next(), Some(Token::Number(123.0)));
        assert_eq!(l.next(), Some(Token::Dot));
        assert_eq!(l.next(), None);
    }
}

fn main() {
    println!("Hello, world!");
}
