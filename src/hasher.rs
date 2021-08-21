use std::{
    collections::hash_map::DefaultHasher,
    hash::{self, BuildHasher},
};

/// Wrapper type for [std::hash::BuildHasherDefault], that uses
/// [DefaultHasher] as the hasher.
#[derive(Clone)]
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

impl Default for BuildHasherDefault {
    fn default() -> Self {
        BuildHasherDefault {
            hasher: hash::BuildHasherDefault::<DefaultHasher>::default(),
        }
    }
}
