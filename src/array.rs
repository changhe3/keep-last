use core::mem::MaybeUninit;

use crate::util::uninit_array;

pub trait Array {
    type Item;
    type Uninit: AsSlice<MaybeUninit<Self::Item>>;

    fn new_uninit(capacity: usize) -> Self::Uninit;
}

pub trait AsSlice<T> {
    fn as_slice(&self) -> &[T];

    fn as_mut_slice(&mut self) -> &mut [T];

    #[inline]
    fn len(&self) -> usize {
        self.as_slice().len()
    }
}

pub trait StaticArray: Array {}

macro_rules! impl_slice {
    ($name:ty, $gen:ident; $($args:tt)*) => {
        impl<$($args)*> $crate::array::AsSlice<$gen> for $name {
            #[inline]
            fn as_slice(&self) -> &[T] {
                self.as_ref()
            }

            #[inline]
            fn as_mut_slice(&mut self) -> &mut [T] {
                self.as_mut()
            }
        }
    };
}

impl_slice!([T; N], T; T, const N: usize);

impl<T, const N: usize> Array for [T; N] {
    type Item = T;

    type Uninit = [MaybeUninit<T>; N];

    fn new_uninit(_: usize) -> Self::Uninit {
        uninit_array()
    }
}

impl<T, const N: usize> StaticArray for [T; N] {}

#[cfg(feature = "alloc")]
mod alloc {
    use core::mem::MaybeUninit;

    use alloc::{boxed::Box, vec::Vec};
    use uninit::prelude::VecCapacity;

    use crate::util::uninit_array;

    use super::{Array, StaticArray};

    impl_slice!(alloc::boxed::Box<[T; N]>, T; T, const N: usize);
    impl_slice!(alloc::boxed::Box<[T]>, T; T);

    impl<T, const N: usize> Array for Box<[T; N]> {
        type Item = T;

        type Uninit = Box<[MaybeUninit<T>; N]>;

        fn new_uninit(_: usize) -> Self::Uninit {
            Box::new(uninit_array())
        }
    }

    impl<T, const N: usize> StaticArray for Box<[T; N]> {}

    impl<T> Array for Box<[T]> {
        type Item = T;

        type Uninit = Box<[MaybeUninit<T>]>;

        fn new_uninit(capacity: usize) -> Self::Uninit {
            Vec::with_capacity(capacity).into_backing_buffer_forget_elems()
        }
    }
}
