//! Library implements xor-filter.
//!
//! This is a port of its
//! [original implementation](https://github.com/FastFilter/xorfilter)
//! written in golang.

mod hasher;
mod xor8;

pub use hasher::BuildHasherDefault;
pub use xor8::Xor8;
