use crate::iter::{At, Buffer, Consumer, Item};

pub struct SliceConsumer<'a, T> {
    iter: &'a [T],
    index: usize,
    at: At,
    next_at: Box<dyn Fn(At, &T) -> At>,
}

impl<'a, T> SliceConsumer<'a, T> {
    fn new(value: &'a [T], next_at: Box<dyn Fn(At, &T) -> At>) -> Self {
        Self {
            iter: value,
            index: 0,
            at: At::new(1, 0),
            next_at,
        }
    }
}

impl<'a, T> Consumer<'a, T> for SliceConsumer<'a, T> {
    fn at(&self) -> At {
        self.at
    }

    fn peek(&mut self) -> Option<Item<T>> {
        if self.index >= self.iter.len() {
            None
        } else {
            Some(Item::Ref(&self.iter[self.index]))
        }
    }

    fn next(&mut self) -> Option<Item<T>> {
        if self.index >= self.iter.len() {
            return None;
        }

        let r = &self.iter[self.index];
        
        self.at = (self.next_at)(self.at, &self.iter[self.index]);
        self.index += 1;
        
        Some(Item::Ref(r))
    }

    fn next_if<F>(&mut self, mut predicate: F) -> Option<Item<T>>
    where
        F: FnMut(&T) -> bool,
        Self: Sized
    {
        if self.index >= self.iter.len() {
            return None;
        }
        
        if predicate(&self.iter[self.index]) {
            self.next().unwrap();
            Some(Item::Ref(&self.iter[self.index]))
        } else {
            None
        }
    }

    fn consume<F>(&mut self, mut predicate: F) -> Buffer<T>
    where
        F: FnMut(&T, usize) -> bool,
        Self: Sized
    {
        let begin = self.index;

        let mut i = 0;
        while let Some(peek) = self.peek() {
            if !predicate(&peek, i) {
                break;
            }

            self.next().unwrap();

            i += 1;
        }

        self.iter[begin..self.index].into()
    }

    fn consume_while<E, F>(&mut self, f: F) -> Result<Buffer<T>, E>
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
        Self: Sized
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
