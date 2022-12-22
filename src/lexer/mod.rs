use lazy_static::lazy_static;
use std::collections::HashMap;

mod cursor;
use cursor::Cursor;
pub use cursor::Loc;

lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, TokKind> = HashMap::from([
        ("and", TokKind::And),
        ("class", TokKind::Class),
        ("else", TokKind::Else),
        ("false", TokKind::False),
        ("fn", TokKind::Fn),
        ("for", TokKind::For),
        ("if", TokKind::If),
        ("let", TokKind::Let),
        ("nil", TokKind::Nil),
        ("or", TokKind::Or),
        ("print", TokKind::Print),
        ("return", TokKind::Return),
        ("this", TokKind::This),
        ("true", TokKind::True),
        ("while", TokKind::While),
    ]);
}

#[derive(PartialEq, Debug)]
pub struct Token {
    pub kind: TokKind,
    pub loc: Loc,
}

#[derive(PartialEq, Debug, Clone)]
pub enum TokKind {
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
pub struct Lexer<I>
where
    I: Iterator<Item = char>,
    I: Clone,
{
    cursor: Cursor<I>,
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
            cursor: Cursor::new(src),
            buf: String::with_capacity(Self::BUF_CAP),
        }
    }

    fn buf_while(&mut self, mut f: impl FnMut(char) -> bool) {
        while let Some(c) = self.cursor.next_if(&mut f) {
            self.buf.push(c);
        }
    }

    fn next_raw(&mut self) -> Option<Token> {
        self.cursor.eat_while(char::is_whitespace);

        use TokKind::*;
        let loc = self.cursor.loc();
        let kind = self.cursor.next().map(|c| match c {
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
                if self.cursor.next_if(|c| c == '=').is_some() {
                    BangEqual
                } else {
                    Bang
                }
            }
            '=' => {
                if self.cursor.next_if(|c| c == '=').is_some() {
                    EqualEqual
                } else {
                    Equal
                }
            }
            '<' => {
                if self.cursor.next_if(|c| c == '=').is_some() {
                    LessEqual
                } else {
                    Less
                }
            }
            '>' => {
                if self.cursor.next_if(|c| c == '=').is_some() {
                    GreaterEqual
                } else {
                    Greater
                }
            }
            '/' => {
                if self.cursor.next_if(|c| c == '/').is_some() {
                    self.cursor.eat_while(|c| c != '\n');
                    Comment
                } else {
                    Slash
                }
            }
            '"' => {
                self.buf.clear();
                self.buf_while(|c| c != '"');
                if self.cursor.next_if(|c| c == '"').is_some() {
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
                if self.cursor.peek().is_some_and(|c| c == '.')
                    && (self.cursor.peek_snd().is_some_and(|c| c.is_ascii_digit()))
                {
                    self.cursor.next();
                    self.buf.push('.');
                    self.buf_while(|c| c.is_ascii_digit())
                }
                Number(self.buf.parse().unwrap())
            }
            _ => Unexpected,
        });

        kind.map(|kind| Token { kind, loc })
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
            if t.kind != TokKind::Comment {
                return Some(t);
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! tok {
        ($k:tt($a:expr), $r:expr, $c:expr) => {
            Token {
                kind: TokKind::$k($a),
                loc: Loc { row: $r, col: $c },
            }
        };
        ($k:tt , $r:expr, $c:expr) => {
            Token {
                kind: TokKind::$k,
                loc: Loc { row: $r, col: $c },
            }
        };
    }

    #[test]
    fn test_lexer_punctuation() {
        let mut l = Lexer::new("(){};,+-*!===<=>=!=<>/.".chars());
        assert_eq!(l.next(), Some(tok!(LParen, 0, 0)));
        assert_eq!(l.next(), Some(tok!(RParen, 0, 1)));
        assert_eq!(l.next(), Some(tok!(LBrace, 0, 2)));
        assert_eq!(l.next(), Some(tok!(RBrace, 0, 3)));
        assert_eq!(l.next(), Some(tok!(Semicolon, 0, 4)));
        assert_eq!(l.next(), Some(tok!(Comma, 0, 5)));
        assert_eq!(l.next(), Some(tok!(Plus, 0, 6)));
        assert_eq!(l.next(), Some(tok!(Minus, 0, 7)));
        assert_eq!(l.next(), Some(tok!(Star, 0, 8)));
        assert_eq!(l.next(), Some(tok!(BangEqual, 0, 9)));
        assert_eq!(l.next(), Some(tok!(EqualEqual, 0, 11)));
        assert_eq!(l.next(), Some(tok!(LessEqual, 0, 13)));
        assert_eq!(l.next(), Some(tok!(GreaterEqual, 0, 15)));
        assert_eq!(l.next(), Some(tok!(BangEqual, 0, 17)));
        assert_eq!(l.next(), Some(tok!(Less, 0, 19)));
        assert_eq!(l.next(), Some(tok!(Greater, 0, 20)));
        assert_eq!(l.next(), Some(tok!(Slash, 0, 21)));
        assert_eq!(l.next(), Some(tok!(Dot, 0, 22)));
        assert_eq!(l.next(), None);
    }

    #[test]
    fn test_lexer_strings() {
        let mut l = Lexer::new(r#"  "string"  ""  "msg" "#.chars());
        assert_eq!(l.next(), Some(tok!(String("string".to_string()), 0, 2)));
        assert_eq!(l.next(), Some(tok!(String("".to_string()), 0, 12)));
        assert_eq!(l.next(), Some(tok!(String("msg".to_string()), 0, 16)));
        assert_eq!(l.next(), None)
    }

    #[test]
    fn test_lexer_identifiers() {
        let mut l = Lexer::new(
            "andy formless fo _ _123 _abc ab123
    abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890_"
                .chars(),
        );
        assert_eq!(l.next(), Some(tok!(Ident("andy".to_string()), 0, 0)));
        assert_eq!(l.next(), Some(tok!(Ident("formless".to_string()), 0, 5)));
        assert_eq!(l.next(), Some(tok!(Ident("fo".to_string()), 0, 14)));
        assert_eq!(l.next(), Some(tok!(Ident("_".to_string()), 0, 17)));
        assert_eq!(l.next(), Some(tok!(Ident("_123".to_string()), 0, 19)));
        assert_eq!(l.next(), Some(tok!(Ident("_abc".to_string()), 0, 24)));
        assert_eq!(l.next(), Some(tok!(Ident("ab123".to_string()), 0, 29)));
        assert_eq!(
            l.next(),
            Some(tok!(
                Ident(
                    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890_".to_string()
                ),
                1,
                4
            ))
        );
        assert_eq!(l.next(), None);
    }

    #[test]
    fn test_lexer_numbers() {
        let mut l = Lexer::new("123 123.456 .456 123.".chars());
        assert_eq!(l.next(), Some(tok!(Number(123.0), 0, 0)));
        assert_eq!(l.next(), Some(tok!(Number(123.456), 0, 4)));
        assert_eq!(l.next(), Some(tok!(Dot, 0, 12)));
        assert_eq!(l.next(), Some(tok!(Number(456.0), 0, 13)));
        assert_eq!(l.next(), Some(tok!(Number(123.0), 0, 17)));
        assert_eq!(l.next(), Some(tok!(Dot, 0, 20)));
        assert_eq!(l.next(), None);
    }

    #[test]
    fn test_lexer_whitespace() {
        let mut l = Lexer::new("space    tabs\t\t\t\tnewline\n\n\nend\r\n".chars());
        assert_eq!(l.next(), Some(tok!(Ident("space".to_string()), 0, 0)));
        assert_eq!(l.next(), Some(tok!(Ident("tabs".to_string()), 0, 9)));
        assert_eq!(l.next(), Some(tok!(Ident("newline".to_string()), 0, 17)));
        assert_eq!(l.next(), Some(tok!(Ident("end".to_string()), 3, 0)));
        assert_eq!(l.next(), None);
    }

    #[test]
    fn test_lexer_keywords() {
        let mut l = Lexer::new(
            "and class else false fn for if
let nil or print return this true while"
                .chars(),
        );
        assert_eq!(l.next(), Some(tok!(And, 0, 0)));
        assert_eq!(l.next(), Some(tok!(Class, 0, 4)));
        assert_eq!(l.next(), Some(tok!(Else, 0, 10)));
        assert_eq!(l.next(), Some(tok!(False, 0, 15)));
        assert_eq!(l.next(), Some(tok!(Fn, 0, 21)));
        assert_eq!(l.next(), Some(tok!(For, 0, 24)));
        assert_eq!(l.next(), Some(tok!(If, 0, 28)));
        assert_eq!(l.next(), Some(tok!(Let, 1, 0)));
        assert_eq!(l.next(), Some(tok!(Nil, 1, 4)));
        assert_eq!(l.next(), Some(tok!(Or, 1, 8)));
        assert_eq!(l.next(), Some(tok!(Print, 1, 11)));
        assert_eq!(l.next(), Some(tok!(Return, 1, 17)));
        assert_eq!(l.next(), Some(tok!(This, 1, 24)));
        assert_eq!(l.next(), Some(tok!(True, 1, 29)));
        assert_eq!(l.next(), Some(tok!(While, 1, 34)));
        assert_eq!(l.next(), None);
    }

    #[test]
    fn test_lexer_comments() {
        let mut l1 = Lexer::new(
            "foo\n// this is a comment\nbar // another comment\n// third comment\nend".chars(),
        );
        let mut l2 = l1.clone();

        assert_eq!(l1.next_raw(), Some(tok!(Ident("foo".to_string()), 0, 0)));
        assert_eq!(l1.next_raw(), Some(tok!(Comment, 1, 0)));
        assert_eq!(l1.next_raw(), Some(tok!(Ident("bar".to_string()), 2, 0)));
        assert_eq!(l1.next_raw(), Some(tok!(Comment, 2, 4)));
        assert_eq!(l1.next_raw(), Some(tok!(Comment, 3, 0)));
        assert_eq!(l1.next_raw(), Some(tok!(Ident("end".to_string()), 4, 0)));
        assert_eq!(l1.next_raw(), None);

        assert_eq!(l2.next(), Some(tok!(Ident("foo".to_string()), 0, 0)));
        assert_eq!(l2.next(), Some(tok!(Ident("bar".to_string()), 2, 0)));
        assert_eq!(l2.next(), Some(tok!(Ident("end".to_string()), 4, 0)));
        assert_eq!(l2.next(), None);
    }

    #[test]
    fn test_lexer_errors() {
        let mut l = Lexer::new(
            r#" foo(bar @ ) "string
true and 1 == 1 "#
                .chars(),
        );
        assert_eq!(l.next(), Some(tok!(Ident("foo".to_string()), 0, 1)));
        assert_eq!(l.next(), Some(tok!(LParen, 0, 4)));
        assert_eq!(l.next(), Some(tok!(Ident("bar".to_string()), 0, 5)));
        assert_eq!(l.next(), Some(tok!(Unexpected, 0, 9)));
        assert_eq!(l.next(), Some(tok!(RParen, 0, 11)));
        assert_eq!(l.next(), Some(tok!(Unterminated, 0, 13)));
    }
}
