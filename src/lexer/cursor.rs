use std::iter::Peekable;

#[derive(Debug, Clone, Copy)]
pub struct Loc {
    pub row: usize,
    pub col: usize,
}

#[derive(Clone)]
pub struct Cursor<I>
where
    I: Iterator<Item = char>,
    I: Clone,
{
    src: Peekable<I>,
    col: usize,
    row: usize,
    bol: usize,
}

impl<I> Cursor<I>
where
    I: Iterator<Item = char>,
    I: Clone,
{
    pub fn new(src: I) -> Self {
        Self {
            src: src.peekable(),
            col: 0,
            row: 0,
            bol: 0,
        }
    }

    pub fn next_if(&mut self, f: impl FnOnce(&char) -> bool) -> Option<char> {
        // Use self.next since it updates loc information
        if self.src.peek().is_some_and(f) {
            self.next()
        } else {
            None
        }
    }

    pub fn eat_while(&mut self, mut f: impl FnMut(char) -> bool) {
        while self.next_if(|&c| f(c)).is_some() { /* spin */ }
    }

    pub fn peek(&mut self) -> Option<char> {
        self.src.peek().copied()
    }

    pub fn peek_snd(&mut self) -> Option<char> {
        let mut it = self.src.clone();
        it.next();
        it.next()
    }
}

impl<I> Iterator for Cursor<I>
where
    I: Iterator<Item = char>,
    I: Clone,
{
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.src.next().map(|x| {
            self.col += 1;
            if x == '\n' {
                self.row += 1;
                self.bol = self.col;
            }
            x
        })
    }
}
