#![no_std]

mod array;
mod keep_last;
mod util;

#[cfg(feature = "alloc")]
extern crate alloc;

pub use array::Array;
use array::StaticArray;
pub use keep_last::KeepLast;

pub trait KeepLastExt: Iterator {
    fn keep_last(self) -> KeepLast<Self, [Self::Item; 1]>
    where
        Self: Sized,
    {
        self.keep_last_with()
    }

    fn keep_last_with<B>(self) -> KeepLast<Self, B>
    where
        B: Array<Item = Self::Item> + StaticArray,
        Self: Sized,
    {
        KeepLast::new_static(self)
    }

    fn keep_last_n<B>(self, capacity: usize) -> KeepLast<Self, B>
    where
        B: Array<Item = Self::Item>,
        Self: Sized,
    {
        KeepLast::new(self, capacity)
    }
}

impl<I: Iterator> KeepLastExt for I {}
