//! Library implements xor-filter.
//!
//! This is a port of its
//! [original implementation](https://github.com/FastFilter/xorfilter)
//! written in golang.

#[allow(unused_imports)]
use std::collections::hash_map::DefaultHasher;
#[allow(unused_imports)]
use std::collections::hash_map::RandomState;
use std::convert::TryInto;
use std::ffi;
use std::fs;
use std::hash::BuildHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::io::{self};
use std::sync::Arc;

#[cfg(feature = "cbordata")]
use cbordata::Cbor;
#[cfg(feature = "cbordata")]
use cbordata::Cborize;
#[cfg(feature = "cbordata")]
use cbordata::FromCbor;
#[cfg(feature = "cbordata")]
use cbordata::IntoCbor;
#[cfg(feature = "cbordata")]
use cbordata::{self as cbor};

use crate::BuildHasherDefault;

pub(in crate::xor8) fn murmur64(mut h: u64) -> u64 {
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    h ^= h >> 33;
    h
}

// returns random number, modifies the seed
pub(in crate::xor8) fn splitmix64(seed: &mut u64) -> u64 {
    *seed = (*seed).wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *seed;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

pub(in crate::xor8) fn mixsplit(key: u64, seed: u64) -> u64 {
    murmur64(key.wrapping_add(seed))
}

pub(in crate::xor8) fn reduce(hash: u32, n: u32) -> u32 {
    // http://lemire.me/blog/2016/06/27/a-fast-alternative-to-the-modulo-reduction/
    (((hash as u64) * (n as u64)) >> 32) as u32
}

pub(in crate::xor8) fn fingerprint(hash: u64) -> u64 {
    hash ^ (hash >> 32)
}

#[derive(Clone, Default)]
pub(in crate::xor8) struct XorSet {
    pub(in crate::xor8) xor_mask: u64,
    pub(in crate::xor8) count: u32,
}

#[derive(Default)]
pub(in crate::xor8) struct Hashes {
    pub(in crate::xor8) h: u64,
    pub(in crate::xor8) h0: u32,
    pub(in crate::xor8) h1: u32,
    pub(in crate::xor8) h2: u32,
}

/// Type Xor8 is probabilistic data-structure to test membership of an element in a set.
///
/// This implementation has a false positive rate of about 0.3% and a memory usage of
/// less than 9 bits per entry for sizeable sets.
///
/// Xor8 is parametrized over type `H` which is expected to implement [BuildHasher]
/// trait, like types [RandomState] and [BuildHasherDefault]. When not supplied,
/// [BuildHasherDefault] is used as the default hash-builder.
///
/// If `RandomState` is used as BuildHasher, `std` has got this to say
/// > _A particular instance RandomState will create the same instances
/// > of Hasher, but the hashers created by two different RandomState_
/// > instances are unlikely to produce the same result for the same values._
///
/// If [DefaultHasher] is used as BuildHasher, `std` has got this to say,
/// > _The internal algorithm is not specified, and so its hashes
/// > should not be relied upon over releases._
///
/// The default type for parameter `H` might change when a reliable and commonly used
/// BuildHasher type is available.
#[derive(Clone, Debug, Default)]
pub struct Xor8<H = BuildHasherDefault>
where H: BuildHasher
{
    pub hash_builder: H,
    pub seed: u64,
    // TODO: Keep `Option` for the compatibility with Cbor format.
    //       It is always `Some` since we have moved out Xor8Builder to another struct.
    pub num_keys: Option<usize>,
    pub block_length: u32,
    pub finger_prints: Arc<Vec<u8>>,
}

impl<H> PartialEq for Xor8<H>
where H: BuildHasher
{
    fn eq(&self, other: &Self) -> bool {
        let num_keys = match (self.num_keys, other.num_keys) {
            (Some(a), Some(b)) => a == b,
            (_, _) => true,
        };

        self.seed == other.seed
            && num_keys
            && self.block_length == other.block_length
            && self.finger_prints == other.finger_prints
    }
}

impl<H> Xor8<H>
where H: BuildHasher
{
    pub(crate) fn new(hash_builder: H) -> Self {
        Self {
            hash_builder,
            seed: 0,
            num_keys: None,
            block_length: 0,
            finger_prints: Arc::new(vec![]),
        }
    }
}

impl<H> Xor8<H>
where H: BuildHasher
{
    #[allow(clippy::len_without_is_empty)]
    /// Return the number of keys added/built into the bitmap index.
    pub fn len(&self) -> Option<usize> {
        self.num_keys
    }

    /// Contains tell you whether the key is likely part of the set, with false
    /// positive rate.
    pub fn contains<K: ?Sized + Hash>(&self, key: &K) -> bool {
        let hashed_key = {
            let mut hasher = self.hash_builder.build_hasher();
            key.hash(&mut hasher);
            hasher.finish()
        };
        self.contains_digest(hashed_key)
    }

    /// Contains tell you whether the key, as pre-computed digest form, is likely
    /// part of the set, with false positive rate.
    pub fn contains_digest(&self, digest: u64) -> bool {
        let hash = mixsplit(digest, self.seed);
        let f = fingerprint(hash) as u8;
        let r0 = hash as u32;
        let r1 = hash.rotate_left(21) as u32;
        let r2 = hash.rotate_left(42) as u32;
        let h0 = reduce(r0, self.block_length) as usize;
        let h1 = (reduce(r1, self.block_length) + self.block_length) as usize;
        let h2 = (reduce(r2, self.block_length) + 2 * self.block_length) as usize;
        f == (self.finger_prints[h0] ^ self.finger_prints[h1] ^ self.finger_prints[h2])
    }

    pub fn get_hasher(&self) -> H::Hasher {
        self.hash_builder.build_hasher()
    }

    /// Calculate hash of a key.
    #[inline]
    pub fn hash<K: Hash + ?Sized>(&self, key: &K) -> u64 {
        let mut hasher = self.get_hasher();
        key.hash(&mut hasher);
        hasher.finish()
    }
}

impl<H> Xor8<H>
where H: BuildHasher
{
    pub(in crate::xor8) fn get_h0h1h2(&self, k: u64) -> Hashes {
        let h = mixsplit(k, self.seed);
        Hashes {
            h,
            h0: reduce(h as u32, self.block_length),
            h1: reduce(h.rotate_left(21) as u32, self.block_length),
            h2: reduce(h.rotate_left(42) as u32, self.block_length),
        }
    }

    pub(in crate::xor8) fn get_h0(&self, hash: u64) -> u32 {
        let r0 = hash as u32;
        reduce(r0, self.block_length)
    }

    pub(in crate::xor8) fn get_h1(&self, hash: u64) -> u32 {
        let r1 = hash.rotate_left(21) as u32;
        reduce(r1, self.block_length)
    }

    pub(in crate::xor8) fn get_h2(&self, hash: u64) -> u32 {
        let r2 = hash.rotate_left(42) as u32;
        reduce(r2, self.block_length)
    }
}

/// Implements serialization and de-serialization logic for Xor8. This is still work
/// in progress, refer to issue: <https://github.com/bnclabs/xorfilter/issues/1>
/// in github.
///
/// TODO: <https://github.com/bnclabs/xorfilter/issues/1>
impl<H> Xor8<H>
where H: Into<Vec<u8>> + From<Vec<u8>> + BuildHasher
{
    /// File signature write on first 4 bytes of file.
    /// ^ stands for xor
    /// TL stands for filter
    /// 1 stands for version 1
    /// 2 stands for version 2
    /// 3 stands for version 3
    const SIGNATURE_V1: [u8; 4] = [b'^', b'T', b'L', 1];
    const SIGNATURE_V2: [u8; 4] = [b'^', b'T', b'L', 2];

    /// METADATA_LENGTH is size that required to write size of all the
    /// metadata of the serialized filter.
    // signature length + seed-length + block-length +
    //      fingerprint-length + hasher-builder length + fingerprint + hash-builder
    const METADATA_LENGTH: usize = 4 + 8 + 4 + 4 + 4;

    /// Write to file in binary format
    /// TODO Add chechsum of finger_prints into file headers
    pub fn write_file(&self, path: &ffi::OsStr) -> io::Result<usize>
    where H: Clone {
        let mut f = fs::File::create(path)?;
        let buf = self.to_bytes();
        f.write_all(&buf)?;
        Ok(buf.len())
    }

    /// Read from file in binary format
    pub fn read_file(path: &ffi::OsStr) -> io::Result<Self>
    where H: Default {
        let mut f = fs::File::open(path)?;
        let mut data = Vec::new();
        f.read_to_end(&mut data)?;
        Self::from_bytes(data)
    }

    pub fn to_bytes(&self) -> Vec<u8>
    where H: Clone {
        let capacity = Self::METADATA_LENGTH + self.finger_prints.len();
        let mut buf: Vec<u8> = Vec::with_capacity(capacity);
        buf.extend_from_slice(&Xor8::<H>::SIGNATURE_V2);
        buf.extend_from_slice(&self.seed.to_be_bytes());
        buf.extend_from_slice(&self.block_length.to_be_bytes());
        buf.extend_from_slice(&(self.finger_prints.len() as u32).to_be_bytes());

        let hb_binary: Vec<u8> = self.hash_builder.clone().into();
        buf.extend_from_slice(&(hb_binary.len() as u32).to_be_bytes());

        buf.extend_from_slice(&self.finger_prints);
        buf.extend_from_slice(&hb_binary);
        buf
    }

    pub fn from_bytes(buf: Vec<u8>) -> io::Result<Self>
    where H: Default {
        use std::io::Error;

        let mut n = 0;

        // validate the buf first.
        if Self::METADATA_LENGTH > buf.len() {
            return Err(Error::new(ErrorKind::InvalidData, "invalid byte slice"));
        }

        // check the signature
        if buf[n..4] == Xor8::<H>::SIGNATURE_V1 {
            return Self::from_bytes_v1(buf);
        } else if buf[n..4] != Xor8::<H>::SIGNATURE_V2 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "File signature incorrect",
            ));
        }

        n += 4;
        // fetch the seed
        let seed = u64::from_be_bytes(buf[n..n + 8].try_into().unwrap());
        n += 8;
        // fetch block_length
        let block_length = u32::from_be_bytes(buf[n..n + 4].try_into().unwrap());
        n += 4;
        // fetch fingerprint length
        let fp_len = u32::from_be_bytes(buf[n..n + 4].try_into().unwrap()) as usize;
        n += 4;
        // fetch hash-serizalized length
        let hb_len = u32::from_be_bytes(buf[n..n + 4].try_into().unwrap()) as usize;
        n += 4;

        if buf[n..].len() < (fp_len + hb_len) {
            return Err(Error::new(ErrorKind::InvalidData, "invalid byte slice"));
        }

        // fetch the finger print
        let finger_prints = Arc::new(buf[n..n + fp_len].to_vec());
        n += fp_len;
        // fetch the hash_builder
        let hash_builder: H = buf[n..n + hb_len].to_vec().into();

        Ok(Xor8 {
            hash_builder,
            seed,
            num_keys: None,
            block_length,
            finger_prints,
        })
    }

    fn from_bytes_v1(buf: Vec<u8>) -> io::Result<Self>
    where H: Default {
        use std::io::Error;

        let fp_len = u32::from_be_bytes(buf[16..20].try_into().unwrap()) as usize;
        if buf[20..].len() < fp_len {
            return Err(Error::new(ErrorKind::InvalidData, "invalid byte slice"));
        }
        Ok(Xor8 {
            hash_builder: H::default(),
            seed: u64::from_be_bytes(buf[4..12].try_into().unwrap()),
            num_keys: None,
            block_length: u32::from_be_bytes(buf[12..16].try_into().unwrap()),
            finger_prints: Arc::new(buf[20..].to_vec()),
        })
    }
}

//------ Implement cbordata related functionalities

// Intermediate type to serialize and de-serialized Xor8 into bytes.
#[cfg(feature = "cbordata")]
#[derive(Cborize)]
struct CborXor8 {
    hash_builder: Vec<u8>,
    seed: u64,
    num_keys: Option<usize>,
    block_length: u32,
    finger_prints: Vec<u8>,
}

#[cfg(feature = "cbordata")]
impl CborXor8 {
    const ID: &'static str = "xor8/0.0.1";
}

#[cfg(feature = "cbordata")]
impl<H> IntoCbor for Xor8<H>
where H: BuildHasher + Into<Vec<u8>>
{
    fn into_cbor(self) -> cbor::Result<Cbor> {
        let val = CborXor8 {
            hash_builder: self.hash_builder.into(),
            seed: self.seed,
            num_keys: self.num_keys,
            block_length: self.block_length,
            finger_prints: self.finger_prints.to_vec(),
        };
        val.into_cbor()
    }
}

#[cfg(feature = "cbordata")]
impl<H> FromCbor for Xor8<H>
where H: BuildHasher + From<Vec<u8>>
{
    fn from_cbor(val: Cbor) -> cbor::Result<Self> {
        let val = CborXor8::from_cbor(val)?;

        let filter = Xor8 {
            hash_builder: val.hash_builder.into(),
            seed: val.seed,
            num_keys: val.num_keys,
            block_length: val.block_length,
            finger_prints: Arc::new(val.finger_prints),
        };

        Ok(filter)
    }
}
