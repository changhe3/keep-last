use core::mem::MaybeUninit;

use uninit::prelude::AsOut;

use crate::{
    array::{Array, AsSlice, StaticArray},
    util::{slice_assume_init_mut, slice_assume_init_ref},
};

pub struct KeepLast<I: Iterator, B: Array<Item = I::Item>> {
    inner: I,
    buf: B::Uninit,
    write_idx: usize,
    backtrack: usize,
}

impl<I: Iterator, B: Array<Item = I::Item>> KeepLast<I, B> {
    #[inline]
    pub(crate) fn new(inner: I, capacity: usize) -> Self {
        Self {
            inner,
            buf: B::new_uninit(capacity),
            write_idx: 0,
            backtrack: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.write_idx.min(self.capacity())
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.write_idx == 0
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    #[inline]
    pub fn backtrack(&mut self, n: usize) {
        self.backtrack_to(self.backtrack.saturating_add(n))
    }

    #[inline]
    pub fn backtrack_to(&mut self, n: usize) {
        self.backtrack = n.min(self.capacity())
    }

    #[inline]
    unsafe fn drop_buffer(&mut self) {
        self.peek_raw_mut()
            .iter_mut()
            .for_each(|ptr| ptr.assume_init_drop())
    }

    #[inline]
    pub fn clear(&mut self) {
        unsafe { self.drop_buffer() };
        self.write_idx = 0;
        self.backtrack = 0;
    }

    #[inline]
    fn peek_raw(&self) -> &[MaybeUninit<I::Item>] {
        &self.buf.as_slice()[..self.len()]
    }

    #[inline]
    fn peek_raw_mut(&mut self) -> &mut [MaybeUninit<I::Item>] {
        let len = self.len();
        &mut self.buf.as_mut_slice()[..len]
    }

    #[inline]
    pub fn peek(&self) -> &[I::Item] {
        unsafe { slice_assume_init_ref(self.peek_raw()) }
    }

    #[inline]
    pub fn peek_mut(&mut self) -> &mut [I::Item] {
        unsafe { slice_assume_init_mut(self.peek_raw_mut()) }
    }
}

impl<I: Iterator, B: StaticArray<Item = I::Item>> KeepLast<I, B> {
    #[inline]
    pub(crate) fn new_static(inner: I) -> Self {
        Self {
            inner,
            buf: B::new_uninit(0),
            write_idx: 0,
            backtrack: 0,
        }
    }
}

impl<I: Iterator, B: Array<Item = I::Item>> Iterator for KeepLast<I, B>
where
    I::Item: Clone,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let buf = self.buf.as_mut_slice();
        let cap = buf.len();

        if self.backtrack == 0 {
            let item = self.inner.next()?;
            let (mut replaced, item) = buf[self.write_idx % cap].as_out().replace(item);
            if self.write_idx >= cap {
                unsafe { replaced.assume_init_drop() };
            }

            self.write_idx = self.write_idx.saturating_add(1);
            Some(item.clone())
        } else {
            let idx = self.write_idx.saturating_sub(self.backtrack);
            self.backtrack -= 1;
            unsafe { Some(buf[idx % cap].assume_init_ref().clone()) }
        }
    }
}

impl<I: Iterator, B: Array<Item = I::Item>> AsRef<[I::Item]> for KeepLast<I, B> {
    #[inline]
    fn as_ref(&self) -> &[I::Item] {
        self.peek()
    }
}

impl<I: Iterator, B: Array<Item = I::Item>> AsMut<[I::Item]> for KeepLast<I, B> {
    #[inline]
    fn as_mut(&mut self) -> &mut [I::Item] {
        self.peek_mut()
    }
}

impl<I: Iterator, B: Array<Item = I::Item>> Drop for KeepLast<I, B> {
    #[inline]
    fn drop(&mut self) {
        self.clear();
    }
}
