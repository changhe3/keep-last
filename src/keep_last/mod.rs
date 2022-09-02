mod drain;

use core::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut, Index, IndexMut, Range, RangeBounds},
};

use uninit::prelude::AsOut;

use crate::{
    array::{Array, AsSlice, StaticArray},
    util::{slice_assume_init_mut, slice_assume_init_ref, slice_range},
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
    pub fn position(&self) -> usize {
        self.backtrack
    }

    #[inline]
    pub fn backtrack(&mut self, n: usize) {
        self.backtrack_to(self.backtrack.saturating_add(n))
    }

    #[inline]
    pub fn backtrack_to(&mut self, n: usize) {
        self.backtrack = n.min(self.len())
    }

    #[inline]
    unsafe fn drop_in_place(&mut self) {
        let buf = self.as_init_slice_mut();
        let ptr = buf as *mut [I::Item];
        ptr.drop_in_place();
    }

    #[inline]
    pub fn clear(&mut self) {
        unsafe { self.drop_in_place() };
        self.write_idx = 0;
        self.backtrack = 0;
    }

    #[inline]
    pub fn as_uninit_slice(&self) -> &[MaybeUninit<I::Item>] {
        self.buf.as_slice()
    }

    #[inline]
    pub fn as_uninit_slice_mut(&mut self) -> &mut [MaybeUninit<I::Item>] {
        self.buf.as_mut_slice()
    }

    #[inline]
    pub fn as_init_slice(&self) -> &[I::Item] {
        unsafe { slice_assume_init_ref(&self.as_uninit_slice()[..self.len()]) }
    }

    #[inline]
    pub fn as_init_slice_mut(&mut self) -> &mut [I::Item] {
        let len = self.len();
        unsafe { slice_assume_init_mut(&mut self.as_uninit_slice_mut()[..len]) }
    }

    #[inline]
    pub fn as_slices(&self) -> (&[I::Item], &[I::Item]) {
        let buf = self.as_init_slice();
        let cap = self.capacity();
        let idx = self.write_idx.saturating_sub(cap) % cap; // 0 if self.write_idx < cap else self.write_idx % cap
        let (tail, head) = buf.split_at(idx);
        (head, tail)
    }

    #[inline]
    pub fn as_mut_slices(&mut self) -> (&mut [I::Item], &mut [I::Item]) {
        let cap = self.capacity();
        let idx = self.write_idx.saturating_sub(cap) % cap; // 0 if self.write_idx < cap else self.write_idx % cap
        let buf = self.as_init_slice_mut();
        let (tail, head) = buf.split_at_mut(idx);
        (head, tail)
    }

    pub fn range<R: RangeBounds<usize>>(&self, range: R) -> (&[I::Item], &[I::Item]) {
        let len = self.len();
        let Range { start, end } = slice_range(range, ..len);

        let (head, tail) = self.as_slices();
        if start < head.len() && end <= head.len() {
            (&head[start..end], &[])
        } else if start < head.len() {
            let end = end - head.len();
            (&head[start..], &tail[..end])
        } else {
            let start = start - head.len();
            let end = end - head.len();
            (&[], &tail[start..end])
        }
    }

    pub fn range_mut<R: RangeBounds<usize>>(
        &mut self,
        range: R,
    ) -> (&mut [I::Item], &mut [I::Item]) {
        let len = self.len();
        let Range { start, end } = slice_range(range, ..len);

        let (head, tail) = self.as_mut_slices();
        if start < head.len() && end <= head.len() {
            (&mut head[start..end], &mut [])
        } else if start < head.len() {
            let end = end - head.len();
            (&mut head[start..], &mut tail[..end])
        } else {
            let start = start - head.len();
            let end = end - head.len();
            (&mut [], &mut tail[start..end])
        }
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

impl<I: Iterator, B: Array<Item = I::Item>> Index<usize> for KeepLast<I, B> {
    type Output = I::Item;

    fn index(&self, index: usize) -> &Self::Output {
        let len = self.len();
        assert!(index < len);

        let (head, tail) = self.as_slices();
        head.get(index)
            .unwrap_or_else(|| unsafe { tail.get_unchecked(index - head.len()) })
    }
}

impl<I: Iterator, B: Array<Item = I::Item>> IndexMut<usize> for KeepLast<I, B> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let len = self.len();
        assert!(index < len);

        let (head, tail) = self.as_mut_slices();
        let head_len = head.len();
        head.get_mut(index)
            .unwrap_or_else(|| unsafe { tail.get_unchecked_mut(index - head_len) })
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
        self.as_init_slice()
    }
}

impl<I: Iterator, B: Array<Item = I::Item>> AsMut<[I::Item]> for KeepLast<I, B> {
    #[inline]
    fn as_mut(&mut self) -> &mut [I::Item] {
        self.as_init_slice_mut()
    }
}

impl<I: Iterator, B: Array<Item = I::Item>> Drop for KeepLast<I, B> {
    #[inline]
    fn drop(&mut self) {
        self.clear();
    }
}

#[cfg(test)]
mod test {
    use core::ops::Deref;

    use alloc::{boxed::Box, vec::Vec};

    use crate::KeepLastExt;

    #[test]
    fn test_partial_filled() {
        let arr = (0..20).collect::<Vec<_>>();
        let mut iter = arr.iter().copied().map(Box::new).keep_last_with::<[_; 5]>();

        assert_eq!(iter.next().as_deref(), Some(&0));
        assert_eq!(iter.next().as_deref(), Some(&1));
        assert_eq!(iter.next().as_deref(), Some(&2));

        iter.clear();

        assert_eq!(iter.next().as_deref(), Some(&3));
        assert_eq!(iter.next().as_deref(), Some(&4));
        assert_eq!(iter.next().as_deref(), Some(&5));
    }

    #[test]
    fn test_keep_last() {
        let arr = (0..20).collect::<Vec<_>>();

        let mut iter = arr.iter().copied().map(Box::new);
        let mut keep_last = iter.by_ref().take(15).keep_last_with::<[_; 5]>();

        // normal iteration:

        assert_eq!(keep_last.next().as_deref(), Some(&0));
        assert_eq!(keep_last.next().as_deref(), Some(&1));
        assert_eq!(keep_last.next().as_deref(), Some(&2));

        assert!(keep_last
            .as_init_slice()
            .iter()
            .map(Deref::deref)
            .copied()
            .eq(0..3));

        // backtrack

        keep_last.backtrack(2);

        assert_eq!(keep_last.next().as_deref(), Some(&1));
        assert_eq!(keep_last.next().as_deref(), Some(&2));

        keep_last.as_mut()[0] = Box::new(-1);

        keep_last.backtrack(5);
        assert_eq!(keep_last.position(), 3);

        assert_eq!(keep_last.next().as_deref(), Some(&-1));
        assert_eq!(keep_last.next().as_deref(), Some(&1));
        assert_eq!(keep_last.next().as_deref(), Some(&2));

        for i in 3..10 {
            assert_eq!(keep_last.next().as_deref(), Some(&i));
        }

        assert!(keep_last
            .as_init_slice()
            .iter()
            .map(Deref::deref)
            .copied()
            .eq(5..10));

        keep_last.backtrack(5);
        assert_eq!(keep_last.position(), 5);
        assert!(keep_last.map(|boxed| *boxed).eq(5..15));

        iter.map(|boxed| *boxed).eq(15..20);
    }
}
