use crate::iter::Consume;
use crate::iter::{Buffer, Consumer};
use std::collections::HashSet;
use std::hash::Hash;
use std::ops::Range;

#[derive(Debug)]
pub enum Pattern<'a, T: Eq + Hash> {
    Literal(Buffer<'a, T>),
    Class(Class<T>),
    Group(Vec<Pattern<'a, T>>),
    Or(Box<Pattern<'a, T>>, Box<Pattern<'a, T>>),
    Repeat(Repeat<'a, T>),
}

#[derive(Debug)]
pub struct Class<T> {
    items: HashSet<T>,
    inclusive: bool,
}

impl<T> Class<T> {
    pub fn new(items: HashSet<T>, inclusive: bool) -> Self {
        Self { items, inclusive }
    }
}

#[derive(Debug)]
pub struct Repeat<'a, T: Eq + Hash> {
    pattern: Box<Pattern<'a, T>>,
    range: Range<usize>,
}

impl<'a, T: Eq + Hash> Repeat<'a, T> {
    pub fn new(pattern: Pattern<'a, T>, range: Range<usize>) -> Self {
        Self {
            pattern: Box::new(pattern),
            range,
        }
    }
}

macro_rules! consumer {
    ({ $($name:tt)+ } |&$self:ident, $iter:ident| $($tt:tt)+) => {
        impl<'a, Item: Eq + Hash + 'static> Consume<'a, Item> for $($name)+ {
            fn consume<T: Consumer<'a, Item> + Sized>(&$self, $iter: &mut T) -> bool {
                $($tt)+
            }
        }
    };
}

consumer!({ Class<Item> } |&self, v| {
    v.next_if(|item| self.items.contains(item) == self.inclusive).is_some()
});

consumer!({ Vec<Pattern<'a, Item>> } |&self, v| {
    let result = v.transact(|v| {
        for pattern in self {
            if !pattern.consume(v) {
                return Err(());
            }
        }
        Ok(())
    });

    result.is_ok()
});

consumer!({ Repeat<'a, Item> } |&self, v| {
    let result = v.transact(|v| {
        for _ in 0..self.range.start {
            if !self.pattern.consume(v) {
                return Err(());
            }
        }
        Ok(())
    });

    if result.is_err() {
        return false;
    }

    for _ in self.range.start..self.range.end {
        if !self.pattern.consume(v) {
            break;
        }
    }

    true
});

consumer!({ Pattern<'a, Item> } |&self, v| {
    match self {
        Pattern::Literal(ch) => ch.consume(v),
        Pattern::Class(class) => class.consume(v),
        Pattern::Group(group) => group.consume(v),
        Pattern::Or(a, b) => a.consume(v) || b.consume(v),
        Pattern::Repeat(repeat) => repeat.consume(v),
    }
});
