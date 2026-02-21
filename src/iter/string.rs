use crate::iter::{At, Buffer, Consumer, Item};
use std::ops::Deref;

pub struct StringConsumer<'a> {
    iter: &'a str,
    index: usize,
    at: At,
}

impl<'a> From<&'a str> for StringConsumer<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            iter: value,
            index: 0,
            at: At::new(1, 0),
        }
    }
}

impl<'a> Consumer<'a, char> for StringConsumer<'a> {
    fn at(&self) -> At {
        self.at
    }

    fn peek(&mut self) -> Option<Item<'a, char>> {
        self.iter[self.index..].chars().next().map(Item::Val)
    }

    fn next(&mut self) -> Option<Item<'a, char>> {
        let Some(current) = self.iter[self.index..].chars().next() else {
            return None;
        };

        self.index += 1;
        if current == '\n' {
            self.at = At::new(self.at.line + 1, 0);
        } else {
            self.at = At::new(self.at.line, self.at.offset + 1);
        }

        Some(Item::Val(current))
    }

    fn next_if<F>(&mut self, mut predicate: F) -> Option<Item<'a, char>>
    where
        F: FnMut(&char) -> bool,
        Self: Sized,
    {
        let ch = *self.peek()?.deref();
        if predicate(&ch) {
            self.next();
            Some(Item::Val(ch))
        } else {
            None
        }
    }

    fn consume<F>(&mut self, mut predicate: F) -> Buffer<'a, char>
    where
        F: FnMut(&char, usize) -> bool,
        Self: Sized,
    {
        let begin = self.index;

        let mut i = 0;
        while let Some(peek) = self.peek() {
            if !predicate(&peek, i) {
                break;
            }

            self.next();

            i += 1;
        }

        self.iter[begin..self.index].into()
    }

    fn consume_while<E, F>(&mut self, f: F) -> Result<Buffer<'a, char>, E>
    where
        F: FnOnce(&mut Self) -> Result<(), E>,
        Self: Sized
    {
        let begin = self.index;
        self.transact(f)?;
        Ok(self.iter[begin..self.index].into())
    }

    fn transact<Q, E, F>(&mut self, f: F) -> Result<Q, E>
    where
        F: FnOnce(&mut Self) -> Result<Q, E>,
        Self: Sized,
    {
        let index = self.index;
        let at = self.at;

        let result = f(self);
        if result.is_err() {
            self.index = index;
            self.at = at;
        }

        result
    }
}
