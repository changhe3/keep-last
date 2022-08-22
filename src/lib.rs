#![no_std]

mod array;
mod keep_last;
mod util;

#[cfg(feature = "alloc")]
extern crate alloc;

pub use array::Array;
pub use keep_last::KeepLast;
