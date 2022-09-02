use core::{
    mem::replace,
    ops::{Range, RangeBounds},
    ptr::NonNull,
    slice,
};

use crate::{util::slice_range, Array, KeepLast};

pub struct Drain<'a, I: Iterator, B: Array<Item = I::Item>> {
    inner: NonNull<KeepLast<I, B>>,
    iter: slice::Iter<'a, I::Item>,
    tail: usize,
    tail_len: usize,
}

impl<'a, I: Iterator, B: Array<Item = I::Item>> Drain<'a, I, B> {
    pub(crate) fn new<R: RangeBounds<usize>>(inner: &'a mut KeepLast<I, B>, r: R) -> Self {
        let len = inner.len();
        let Range { start, end } = slice_range(r, ..len);

        let old_write_idx = replace(&mut inner.write_idx, start);
        let old_bacKtrack_idx = replace(&mut inner.backtrack, 0);
        todo!()
    }
}
