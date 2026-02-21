mod buffer;
mod iterator;
mod slice;
mod string;
mod at;
mod consume;
mod item;

pub use crate::iter::buffer::Buffer;
pub use crate::iter::item::Item;

pub use crate::iter::iterator::IteratorConsumer;
pub use crate::iter::slice::SliceConsumer;
pub use crate::iter::string::StringConsumer;

pub use at::At;
pub use consume::Consume;

pub trait Consumer<'a, T: 'a> {
    fn at(&self) -> At;

    fn peek(&mut self) -> Option<Item<T>>;

    fn next(&mut self) -> Option<Item<T>>;

    fn next_if<F>(&mut self, predicate: F) -> Option<Item<T>>
    where
        F: FnMut(&T) -> bool,
        Self: Sized;

    fn consume<F>(&mut self, predicate: F) -> Buffer<'a, T>
    where
        F: FnMut(&T, usize) -> bool,
        Self: Sized;
    
    fn consume_while<E, F>(&mut self, f: F) -> Result<Buffer<'a, T>, E>
    where
        F: FnOnce(&mut Self) -> Result<(), E>,
        Self: Sized;

    fn transact<Q, E, F>(&mut self, f: F) -> Result<Q, E>
    where
        F: FnOnce(&mut Self) -> Result<Q, E>,
        Self: Sized;

    fn eof(&mut self) -> bool {
        self.peek().is_none()
    }
}
