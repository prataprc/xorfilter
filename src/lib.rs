#![allow(clippy::bool_to_int_with_if)]

//! Library implements xor-filter.
//!
//! Refer to original implementation under `github.com/FastFilter` to learn the
//! differences between [Xor8], [Fuse8] and [Fuse16] filters. Otherwise, all the types
//! provides similar methods.
//!
//! Starting from version `0.6.0` [Xor8] type is split into [xor8::Xor8] and
//! [xor8::Xor8Builder] under module [xor8]. And [Xor8] type is now deprecated.
//!
//! Provides hasher types:
//!
//! All filter-types are parametrised over user supplied hasher-type.
//!
//! * Use [NoHash] when hash feature is not needed on [Xor8], [Fuse8] and [Fuse16] types.
//!   Note that type methods that accept parametrized key cannot be used.
//! * [BuildHasherDefault] is the default hasher when `H` is not supplied. Note that
//!   [DefaultHasher] uses an unspecified internal algorithm and so its hashes should not
//!   be relied upon over releases.
//!
//! **Handling duplicates**
//!
//! * [Fuse16] and [Xor8] implementation uses BTreeMap to make sure all the digests
//!   generated from keys are unique, this avoids duplicates but decreases the build
//!   performance significantly.
//! * [Fuse8] implementation computes duplicates on the fly leading to significantly
//!   better build performance. On the other hand, Fuse8 cannot handle more than few
//!   duplicates.
//!
//! **Cloning**
//!
//! Cloning [Xor8], [Fuse8], [Fuse16] is fast, but valid only after the filter
//! is constructed. This can linearly scale for read-concurrency with lookup operation.
//!
//! This is ported from its original implementation:
//!
//! **Features**
//!
//! * Enable ``cbordata`` feature for serialize and deserialize [Xor8] [Fuse8] [Fuse16]
//!   types using CBOR spec.
//!
//! * [Xor8] from <https://github.com/FastFilter/xorfilter>, written in golang.
//! * [Fuse8] and [Fuse16] from <https://github.com/FastFilter/xor_singleheader>  written
//!   in C.

#[allow(unused_imports)]
use std::collections::hash_map::DefaultHasher;
use std::error;
use std::fmt;
use std::result;

/// Short form to compose Error values.
///
/// Here are few possible ways:
///
/// ```ignore
/// use crate::Error;
/// err_at!(ParseError, msg: "bad argument");
/// ```
///
/// ```ignore
/// use crate::Error;
/// err_at!(ParseError, std::io::read(buf));
/// ```
///
/// ```ignore
/// use crate::Error;
/// err_at!(ParseError, std::fs::read(file_path), "read failed");
/// ```
macro_rules! err_at {
    ($v:ident, msg: $($arg:expr),+) => {{
        let prefix = format!("{}:{}", file!(), line!());
        Err(Error::$v(prefix, format!($($arg),+)))
    }};
    ($v:ident, $e:expr) => {{
        match $e {
            Ok(val) => Ok(val),
            Err(err) => {
                let prefix = format!("{}:{}", file!(), line!());
                Err(Error::$v(prefix, format!("{}", err)))
            }
        }
    }};
    ($v:ident, $e:expr, $($arg:expr),+) => {{
        match $e {
            Ok(val) => Ok(val),
            Err(err) => {
                let prefix = format!("{}:{}", file!(), line!());
                let msg = format!($($arg),+);
                Err(Error::$v(prefix, format!("{} {}", err, msg)))
            }
        }
    }};
}

/// Error variants that are returned by this package's API.
///
/// Each variant carries a prefix, typically identifying the
/// error location.
pub enum Error {
    Fatal(String, String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        use Error::*;

        match self {
            Fatal(p, msg) => write!(f, "{} Fatal: {}", p, msg),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}

impl error::Error for Error {}

/// Type alias for Result return type, used by this package.
pub type Result<T> = result::Result<T, Error>;

mod fuse16;
mod fuse8;
mod hasher;
mod xor8_old;

pub mod xor8;
pub use fuse16::Fuse16;
pub use fuse8::Fuse8;
pub use hasher::BuildHasherDefault;
pub use hasher::NoHash;
#[deprecated(since = "0.6.0", note = "Use xor8::Xor8 and xor8::Xor8Builder types")]
pub use xor8_old::Xor8;
