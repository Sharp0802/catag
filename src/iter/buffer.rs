use crate::iter::item::Item;
use crate::iter::{Consume, Consumer};
use std::any::TypeId;
use std::mem::transmute_copy;
use std::ops::{Deref, Range};
use std::slice::from_raw_parts;

trait IsChar {
    fn is_char() -> bool;
}

impl<T: 'static> IsChar for T {
    fn is_char() -> bool {
        TypeId::of::<T>() == TypeId::of::<char>()
    }
}

trait Combine {
    fn combine(self, other: Self) -> Self;
}

#[derive(Debug, Hash)]
enum StringRef<'a> {
    Ref(&'a str),
    Val(String)
}

impl Deref for StringRef<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            StringRef::Ref(r) => r,
            StringRef::Val(v) => v,
        }
    }
}

#[derive(Debug, Hash)]
enum BufferImpl<'a, T> {
    Vec(Vec<T>),
    Slice(&'a [T]),
    String(StringRef<'a>),
}

#[derive(Debug, Hash)]
pub struct Buffer<'a, T> {
    inner: BufferImpl<'a, T>,
}

impl<'a> From<&'a str> for Buffer<'a, char> {
    fn from(value: &'a str) -> Self {
        Self {
            inner: BufferImpl::String(StringRef::Ref(value)),
        }
    }
}

impl<'a> From<String> for Buffer<'a, char> {
    fn from(value: String) -> Self {
        Self {
            inner: BufferImpl::String(StringRef::Val(value))
        }
    }
}

impl<'a, T> From<&'a [T]> for Buffer<'a, T> {
    fn from(value: &'a [T]) -> Self {
        Self {
            inner: BufferImpl::Slice(value),
        }
    }
}

impl<'a, T> From<Vec<T>> for Buffer<'a, T> {
    fn from(value: Vec<T>) -> Self {
        Self {
            inner: BufferImpl::Vec(value),
        }
    }
}

impl<'a, T: PartialEq + 'static> PartialEq<[T]> for Buffer<'a, T> {
    fn eq(&self, other: &[T]) -> bool {
        match &self.inner {
            BufferImpl::Vec(vec) => vec == other,
            BufferImpl::Slice(slice) => *slice == other,

            BufferImpl::String(str) => {
                /*
                 * This check is required to suppress undefined behaviour by rust compiler
                 * even if this check is unnecessary (see below for detailed information)
                 */
                if !T::is_char() {
                    return false;
                }

                /*
                 * SAFETY : this branch is only reachable via Buffer<'a, char>::from().
                 * We transmute char to T.
                 * Since we control construction, T is guaranteed to be char here.
                 */
                let transmuted: &[char] = unsafe {
                    from_raw_parts(other.as_ptr() as *const char, other.len())
                };
                transmuted.iter().copied().eq(str.chars())
            },
        }
    }
}

impl<'a> PartialEq<str> for Buffer<'a, char> {
    fn eq(&self, other: &str) -> bool {
        match &self.inner {
            BufferImpl::Vec(vec) => vec.iter().copied().eq(other.chars()),
            BufferImpl::Slice(slice) => slice.iter().copied().eq(other.chars()),
            BufferImpl::String(str) => str.deref() == other,
        }
    }
}

impl<'a, T: PartialEq + 'static> Eq for Buffer<'a, T> {
}

impl<'a, T: PartialEq + 'static> PartialEq for Buffer<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        match &other.inner {
            BufferImpl::Vec(vec) => return self == vec.as_slice(),
            BufferImpl::Slice(slice) => return self == *slice,
            BufferImpl::String(_) => (),
        };

        self.iter().eq(other.iter())
    }
}

impl<'a, T> Buffer<'a, T> {
    pub fn iter(&self) -> Iter<T> {
        match &self.inner {
            BufferImpl::Vec(vec) => Iter::Slice(vec.as_slice().iter()),
            BufferImpl::Slice(slice) => Iter::Slice(slice.iter()),
            BufferImpl::String(str) => Iter::Chars(str.chars()),
        }
    }

    pub fn is_empty(&self) -> bool {
        match &self.inner {
            BufferImpl::Vec(vec) => vec.is_empty(),
            BufferImpl::Slice(slice) => slice.is_empty(),
            BufferImpl::String(str) => str.is_empty(),
        }
    }
}

pub enum Iter<'a, T> {
    Slice(std::slice::Iter<'a, T>),
    Chars(std::str::Chars<'a>),
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = Item<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Iter::Slice(iter) => iter.next().map(Item::Ref),

            Iter::Chars(iter) => {
                /*
                 * SAFETY : this branch is only reachable via Buffer<'a, char>::iter().
                 * We transmute char to T.
                 * Since we control construction, T is guaranteed to be char here.
                 */

                let tmp = iter.next()?;
                let transmuted: T = unsafe { transmute_copy(&tmp) };
                Some(Item::Val(transmuted))
            }
        }
    }
}

impl<'a, Item: PartialEq + 'static> Consume<'a, Item> for Buffer<'a, Item> {
    fn consume<T: Consumer<'a, Item> + Sized>(&self, v: &mut T) -> bool {
        v.transact(|v| {
            let mut iter = self.iter();
            let consumed = v.consume(|item, _| iter.next().as_deref() == Some(item));

            if &consumed == self {
                Ok(())
            } else {
                Err(())
            }
        }).is_ok()
    }
}
