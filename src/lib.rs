//! Library implements xor-filter.
//!
//! This is a port of its
//! [original implementation](https://github.com/FastFilter/xorfilter)
//! written in golang.

mod fuse16;
mod fuse8;
mod hasher;
mod xor8;

pub use fuse16::Fuse16;
pub use fuse8::Fuse8;
pub use hasher::{BuildHasherDefault, NoHash};
pub use xor8::Xor8;
