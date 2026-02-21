use crate::iter::Consumer;

pub trait Consume<'a, Item: 'a> {
    fn consume<T: Consumer<'a, Item> + Sized>(&self, v: &mut T) -> bool;
}
