use crate::iter::{At, Buffer, Consumer, Item};
use std::collections::VecDeque;
use std::ops::Deref;

pub struct IteratorConsumer<'a, T> {
    iter: Box<dyn Iterator<Item = T> + 'a>,
    index: usize,
    saved_at: Option<usize>,
    buffer: VecDeque<T>,

    at: At,
    next_at: Box<dyn Fn(At, &T) -> At>,
}

impl<'a, T> IteratorConsumer<'a, T> {
    pub fn new<I: Iterator<Item = T> + 'a, F: Fn(At, &T) -> At + 'static>(value: I, next_at: F) -> Self {
        Self {
            iter: Box::new(value),
            index: 0,
            saved_at: None,
            buffer: VecDeque::new(),
            at: At::new(1, 0),
            next_at: Box::new(next_at),
        }
    }
}

impl<'a, T: Clone + 'a> Consumer<'a, T> for IteratorConsumer<'a, T> {
    fn at(&self) -> At {
        self.at
    }

    fn peek(&mut self) -> Option<Item<T>> {
        let needs_pull = if let Some(saved_at) = self.saved_at {
            self.buffer.get(self.index - saved_at).is_none()
        } else {
            self.buffer.is_empty()
        };

        if needs_pull {
            if let Some(item) = self.iter.next() {
                self.buffer.push_back(item);
            } else {
                return None;
            }
        }

        if let Some(saved_at) = self.saved_at {
            self.buffer
                .get(self.index - saved_at)
                .or_else(|| self.buffer.back())
        } else {
            self.buffer.front()
        }
        .map(Item::Ref)
    }

    fn next(&mut self) -> Option<Item<T>> {
        let buffer_idx = if let Some(saved_at) = self.saved_at {
            let idx = self.index - saved_at;
            if idx < self.buffer.len() {
                Some(idx)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(idx) = buffer_idx {
            let item = &self.buffer[idx];

            self.index += 1;
            self.at = (self.next_at)(self.at, item);

            return Some(Item::Ref(item));
        }

        if self.saved_at.is_none() {
            if let Some(peeked) = self.buffer.pop_front() {
                self.index += 1;
                self.at = (self.next_at)(self.at, &peeked);

                return Some(Item::Val(peeked));
            }
        }

        if let Some(item) = self.iter.next() {
            self.index += 1;
            self.at = (self.next_at)(self.at, &item);

            if self.saved_at.is_some() {
                self.buffer.push_back(item.clone());
            }

            Some(Item::Val(item))
        } else {
            None
        }
    }

    fn next_if<F>(&mut self, mut predicate: F) -> Option<Item<T>>
    where
        F: FnMut(&T) -> bool,
        Self: Sized,
    {
        if predicate(self.peek()?.deref()) {
            let item = self.next().unwrap();
            Some(item)
        } else {
            None
        }
    }

    fn consume<F>(&mut self, mut predicate: F) -> Buffer<'a, T>
    where
        F: FnMut(&T, usize) -> bool,
        Self: Sized,
    {
        let mut i = 0;
        let mut buffer = Vec::new();
        while let Some(item) = self.peek() {
            if !predicate(&item, i) {
                break;
            }

            let item = self.next().unwrap();
            buffer.push(item.clone());

            i += 1;
        }

        buffer.into()
    }

    fn consume_while<E, F>(&mut self, f: F) -> Result<Buffer<T>, E>
    where
        F: FnOnce(&mut Self) -> Result<(), E>,
        Self: Sized
    {
        let begin = self.index;
        let begin_at = self.at;

        let (is_root_tx, saved_at) = if let Some(saved_at) = self.saved_at {
            (false, saved_at)
        } else {
            self.saved_at = Some(begin);
            (true, begin)
        };

        let result = f(self);

        if is_root_tx {
            self.saved_at = None;
        }

        if let Err(e) = result {
            self.index = begin;
            self.at = begin_at;
            return Err(e);
        }

        let count = self.index - begin;
        let result: Vec<T> = if is_root_tx {
            self.buffer.drain(..count).collect()
        } else {
            self.buffer.iter().skip(begin - saved_at).take(count).cloned().collect()
        };

        Ok(result.into())
    }

    fn transact<Q, E, F>(&mut self, f: F) -> Result<Q, E>
    where
        F: FnOnce(&mut Self) -> Result<Q, E>,
    {
        let cp = self.index;
        let cp_at = self.at;

        let is_root_tx = self.saved_at.is_none();
        if is_root_tx {
            self.saved_at = Some(cp);
        }

        let result = f(self);

        if is_root_tx {
            self.saved_at = None;
        }

        if result.is_err() {
            self.index = cp;
            self.at = cp_at;
        } else if is_root_tx {
            let count = self.index - cp;
            self.buffer.drain(..count);
        }

        result
    }
}
