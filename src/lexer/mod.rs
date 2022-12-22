use lazy_static::lazy_static;
use std::{collections::HashMap, iter::Peekable};

lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, Token> = HashMap::from([
        ("and", Token::And),
        ("class", Token::Class),
        ("else", Token::Else),
        ("false", Token::False),
        ("fn", Token::Fn),
        ("for", Token::For),
        ("if", Token::If),
        ("let", Token::Let),
        ("nil", Token::Nil),
        ("or", Token::Or),
        ("print", Token::Print),
        ("return", Token::Return),
        ("this", Token::This),
        ("true", Token::True),
        ("while", Token::While),
    ]);
}

#[derive(PartialEq, Debug, Clone)]
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
    Unexpected,
    Unterminated,
}

#[derive(Clone)]
pub struct Lexer<I: Iterator<Item = char>> {
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
            '"' => {
                self.buf.clear();
                self.buf_while(|c| c != '"');
                if self.src.next_if(|c| c == &'"').is_some() {
                    String(self.buf.to_string())
                } else {
                    Unterminated
                }
            }
            x if x.is_ascii_alphabetic() || x == '_' => {
                self.buf.clear();
                self.buf.push(x);
                self.buf_while(|c| c.is_ascii_alphanumeric() || c == '_');
                KEYWORDS
                    .get(&*self.buf)
                    .map_or_else(|| Ident(self.buf.to_string()), |kw| kw.clone())
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
            _ => Unexpected,
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
        let mut l = Lexer::new("(){};,+-*!===<=>=!=<>/.".chars());
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
    fn test_lexer_strings() {
        let mut l = Lexer::new(r#"  "string"  ""  "msg" "#.chars());
        assert_eq!(l.next(), Some(Token::String("string".to_string())));
        assert_eq!(l.next(), Some(Token::String("".to_string())));
        assert_eq!(l.next(), Some(Token::String("msg".to_string())));
        assert_eq!(l.next(), None)
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

    #[test]
    fn test_lexer_whitespace() {
        let mut l = Lexer::new("space    tabs\t\t\t\tnewline\n\n\nend\r\n".chars());
        assert_eq!(l.next(), Some(Token::Ident("space".to_string())));
        assert_eq!(l.next(), Some(Token::Ident("tabs".to_string())));
        assert_eq!(l.next(), Some(Token::Ident("newline".to_string())));
        assert_eq!(l.next(), Some(Token::Ident("end".to_string())));
        assert_eq!(l.next(), None);
    }

    #[test]
    fn test_lexer_keywords() {
        let mut l = Lexer::new(
            "and class else false fn for if let nil or print return this true while".chars(),
        );
        assert_eq!(l.next(), Some(Token::And));
        assert_eq!(l.next(), Some(Token::Class));
        assert_eq!(l.next(), Some(Token::Else));
        assert_eq!(l.next(), Some(Token::False));
        assert_eq!(l.next(), Some(Token::Fn));
        assert_eq!(l.next(), Some(Token::For));
        assert_eq!(l.next(), Some(Token::If));
        assert_eq!(l.next(), Some(Token::Let));
        assert_eq!(l.next(), Some(Token::Nil));
        assert_eq!(l.next(), Some(Token::Or));
        assert_eq!(l.next(), Some(Token::Print));
        assert_eq!(l.next(), Some(Token::Return));
        assert_eq!(l.next(), Some(Token::This));
        assert_eq!(l.next(), Some(Token::True));
        assert_eq!(l.next(), Some(Token::While));
    }

    #[test]
    fn test_lexer_comments() {
        let mut l1 = Lexer::new(
            "foo\n// this is a comment\nbar // another comment\n// third comment\nend".chars(),
        );
        let mut l2 = l1.clone();

        assert_eq!(l1.next_raw(), Some(Token::Ident("foo".to_string())));
        assert_eq!(l1.next_raw(), Some(Token::Comment));
        assert_eq!(l1.next_raw(), Some(Token::Ident("bar".to_string())));
        assert_eq!(l1.next_raw(), Some(Token::Comment));
        assert_eq!(l1.next_raw(), Some(Token::Comment));
        assert_eq!(l1.next_raw(), Some(Token::Ident("end".to_string())));
        assert_eq!(l1.next_raw(), None);

        assert_eq!(l2.next(), Some(Token::Ident("foo".to_string())));
        assert_eq!(l2.next(), Some(Token::Ident("bar".to_string())));
        assert_eq!(l2.next(), Some(Token::Ident("end".to_string())));
        assert_eq!(l2.next(), None);
    }

    #[test]
    fn test_lexer_errors() {
        let mut l = Lexer::new(r#"foo(bar @ ) "string true and 1 == 1 "#.chars());
        assert_eq!(l.next(), Some(Token::Ident("foo".to_string())));
        assert_eq!(l.next(), Some(Token::LParen));
        assert_eq!(l.next(), Some(Token::Ident("bar".to_string())));
        assert_eq!(l.next(), Some(Token::Unexpected));
        assert_eq!(l.next(), Some(Token::RParen));
        assert_eq!(l.next(), Some(Token::Unterminated));
    }
}
