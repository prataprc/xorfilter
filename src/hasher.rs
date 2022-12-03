use std::collections::hash_map::DefaultHasher;
use std::hash::BuildHasher;
use std::hash::Hasher;
use std::hash::{self};

/// Wrapper type for [std::hash::BuildHasherDefault], that uses
/// [DefaultHasher] as the hasher.
#[derive(Clone, Default)]
pub struct BuildHasherDefault {
    hasher: hash::BuildHasherDefault<DefaultHasher>,
}

impl From<BuildHasherDefault> for Vec<u8> {
    fn from(_: BuildHasherDefault) -> Vec<u8> {
        vec![]
    }
}

impl From<Vec<u8>> for BuildHasherDefault {
    fn from(_: Vec<u8>) -> BuildHasherDefault {
        BuildHasherDefault {
            hasher: hash::BuildHasherDefault::<DefaultHasher>::default(),
        }
    }
}

impl BuildHasher for BuildHasherDefault {
    type Hasher = DefaultHasher;

    fn build_hasher(&self) -> Self::Hasher {
        self.hasher.build_hasher()
    }
}

/// NoHash type skips hashing altogether.
///
/// When a filter is constructed using NoHash as the type parameter then it is upto
/// application to generate the 64-bit hash digest outside this library.
#[derive(Clone)]
pub struct NoHash;

impl From<NoHash> for Vec<u8> {
    fn from(_: NoHash) -> Vec<u8> {
        vec![]
    }
}

impl From<Vec<u8>> for NoHash {
    fn from(_: Vec<u8>) -> NoHash {
        NoHash
    }
}

impl BuildHasher for NoHash {
    type Hasher = NoHash;

    fn build_hasher(&self) -> Self {
        NoHash
    }
}

impl Default for NoHash {
    fn default() -> Self {
        NoHash
    }
}

impl Hasher for NoHash {
    fn write(&mut self, _bytes: &[u8]) {
        panic!("Can't generate hash digest using NoHash")
    }

    fn finish(&self) -> u64 {
        panic!("Can't generate hash digest using NoHash")
    }
}
