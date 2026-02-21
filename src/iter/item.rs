use std::ops::Deref;

#[derive(Debug)]
pub enum Item<'a, T> {
    Ref(&'a T),
    Val(T),
}

impl<T: PartialEq> PartialEq for Item<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}

impl<'a, T> Deref for Item<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Item::Ref(r) => r,
            Item::Val(v) => v,
        }
    }
}
