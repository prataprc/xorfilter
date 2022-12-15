//! Library implements xor-filter.
//!
//! This is a port of its
//! [original implementation](https://github.com/FastFilter/xorfilter)
//! written in golang.

#[allow(unused_imports)]
use std::collections::hash_map::DefaultHasher;
#[allow(unused_imports)]
use std::collections::hash_map::RandomState;
use std::collections::BTreeMap;
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

use crate::BuildHasherDefault;
use crate::Result;

fn murmur64(mut h: u64) -> u64 {
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    h ^= h >> 33;
    h
}

// returns random number, modifies the seed
fn splitmix64(seed: &mut u64) -> u64 {
    *seed = (*seed).wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *seed;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn mixsplit(key: u64, seed: u64) -> u64 {
    murmur64(key.wrapping_add(seed))
}

fn reduce(hash: u32, n: u32) -> u32 {
    // http://lemire.me/blog/2016/06/27/a-fast-alternative-to-the-modulo-reduction/
    (((hash as u64) * (n as u64)) >> 32) as u32
}

fn fingerprint(hash: u64) -> u64 {
    hash ^ (hash >> 32)
}

#[derive(Clone, Default)]
struct XorSet {
    xor_mask: u64,
    count: u32,
}

#[derive(Default)]
struct Hashes {
    h: u64,
    h0: u32,
    h1: u32,
    h2: u32,
}

#[derive(Clone, Copy, Default)]
struct KeyIndex {
    hash: u64,
    index: u32,
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
pub struct Xor8<H = BuildHasherDefault>
where H: BuildHasher
{
    keys: Option<BTreeMap<u64, ()>>,
    pub hash_builder: H,
    pub seed: u64,
    pub block_length: u32,
    pub finger_prints: Vec<u8>,
}

impl<H> PartialEq for Xor8<H>
where H: BuildHasher
{
    fn eq(&self, other: &Self) -> bool {
        self.seed == other.seed
            && self.block_length == other.block_length
            && self.finger_prints == other.finger_prints
    }
}

impl<H> Default for Xor8<H>
where H: BuildHasher + Default
{
    fn default() -> Self {
        Xor8 {
            keys: Some(BTreeMap::new()),
            hash_builder: H::default(),
            seed: u64::default(),
            block_length: u32::default(),
            finger_prints: Vec::default(),
        }
    }
}

impl<H> Xor8<H>
where H: BuildHasher
{
    /// New Xor8 instance initialized with [DefaultHasher].
    pub fn new() -> Self
    where H: Default {
        Self::default()
    }

    /// New Xor8 instance initialized with supplied `hasher`.
    pub fn with_hasher(hash_builder: H) -> Self {
        Xor8 {
            keys: Some(BTreeMap::new()),
            hash_builder,
            seed: u64::default(),
            block_length: u32::default(),
            finger_prints: Vec::default(),
        }
    }
}

impl<H> Xor8<H>
where H: BuildHasher
{
    /// Insert 64-bit digest of a single key. Digest for the key shall be generated
    /// using the default-hasher or via hasher supplied via [Xor8::with_hasher] method.
    pub fn insert<K: ?Sized + Hash>(&mut self, key: &K) {
        let hashed_key = {
            let mut hasher = self.hash_builder.build_hasher();
            key.hash(&mut hasher);
            hasher.finish()
        };
        self.keys.as_mut().unwrap().insert(hashed_key, ());
    }

    /// Populate with 64-bit digests for a collection of keys of type `K`. Digest for
    /// key shall be generated using the default-hasher or via hasher supplied
    /// via [Xor8::with_hasher] method.
    pub fn populate<K: Hash>(&mut self, keys: &[K]) {
        keys.iter().for_each(|key| {
            let mut hasher = self.hash_builder.build_hasher();
            key.hash(&mut hasher);
            self.keys.as_mut().unwrap().insert(hasher.finish(), ());
        })
    }

    /// Populate with pre-compute collection of 64-bit digests.
    pub fn populate_keys(&mut self, digests: &[u64]) {
        for digest in digests.iter() {
            self.keys.as_mut().unwrap().insert(*digest, ());
        }
    }

    /// Build bitmap for keys that where previously inserted using [Xor8::insert],
    /// [Xor8::populate] and [Xor8::populate_keys] method.
    pub fn build(&mut self) -> Result<()> {
        match self.keys.take() {
            Some(keys) => {
                let digests = keys.keys().copied().collect::<Vec<u64>>();
                self.build_keys(&digests)
            }
            None => Ok(()),
        }
    }

    /// Build a bitmap for pre-computed 64-bit digests for keys. If keys where
    /// previously inserted using [Xor8::insert] or [Xor8::populate] or
    /// [Xor8::populate_keys] methods, they shall be ignored.
    ///
    /// It is upto the caller to ensure that digests are unique, that there no
    /// duplicates.
    pub fn build_keys(&mut self, digests: &[u64]) -> Result<()> {
        let (size, mut rngcounter) = (digests.len(), 1_u64);
        let capacity = {
            let capacity = 32 + ((1.23 * (size as f64)).ceil() as u32);
            capacity / 3 * 3 // round it down to a multiple of 3
        };
        self.seed = splitmix64(&mut rngcounter);
        self.block_length = capacity / 3;
        self.finger_prints = vec![u8::default(); capacity as usize];

        let block_length = self.block_length as usize;
        let mut q0: Vec<KeyIndex> = Vec::with_capacity(block_length);
        let mut q1: Vec<KeyIndex> = Vec::with_capacity(block_length);
        let mut q2: Vec<KeyIndex> = Vec::with_capacity(block_length);
        let mut stack: Vec<KeyIndex> = Vec::with_capacity(size);
        let mut sets0: Vec<XorSet> = vec![XorSet::default(); block_length];
        let mut sets1: Vec<XorSet> = vec![XorSet::default(); block_length];
        let mut sets2: Vec<XorSet> = vec![XorSet::default(); block_length];

        loop {
            for key in digests.iter() {
                let hs = self.geth0h1h2(*key);
                sets0[hs.h0 as usize].xor_mask ^= hs.h;
                sets0[hs.h0 as usize].count += 1;
                sets1[hs.h1 as usize].xor_mask ^= hs.h;
                sets1[hs.h1 as usize].count += 1;
                sets2[hs.h2 as usize].xor_mask ^= hs.h;
                sets2[hs.h2 as usize].count += 1;
            }

            q0.clear();
            q1.clear();
            q2.clear();

            let iter = sets0.iter().enumerate().take(self.block_length as usize);
            for (i, item) in iter {
                if item.count == 1 {
                    q0.push(KeyIndex {
                        index: i as u32,
                        hash: item.xor_mask,
                    });
                }
            }
            let iter = sets1.iter().enumerate().take(self.block_length as usize);
            for (i, item) in iter {
                if item.count == 1 {
                    q1.push(KeyIndex {
                        index: i as u32,
                        hash: item.xor_mask,
                    });
                }
            }
            let iter = sets2.iter().enumerate().take(self.block_length as usize);
            for (i, item) in iter {
                if item.count == 1 {
                    q2.push(KeyIndex {
                        index: i as u32,
                        hash: item.xor_mask,
                    });
                }
            }

            stack.clear();

            while !q0.is_empty() || !q1.is_empty() || !q2.is_empty() {
                while let Some(keyindexvar) = q0.pop() {
                    if sets0[keyindexvar.index as usize].count == 0 {
                        // not actually possible after the initial scan.
                        continue;
                    }
                    let hash = keyindexvar.hash;
                    let h1 = self.geth1(hash);
                    let h2 = self.geth2(hash);
                    stack.push(keyindexvar);

                    let mut s = unsafe { sets1.get_unchecked_mut(h1 as usize) };
                    s.xor_mask ^= hash;
                    s.count -= 1;
                    if s.count == 1 {
                        q1.push(KeyIndex {
                            index: h1,
                            hash: s.xor_mask,
                        })
                    }

                    let mut s = unsafe { sets2.get_unchecked_mut(h2 as usize) };
                    s.xor_mask ^= hash;
                    s.count -= 1;
                    if s.count == 1 {
                        q2.push(KeyIndex {
                            index: h2,
                            hash: s.xor_mask,
                        })
                    }
                }
                while let Some(mut keyindexvar) = q1.pop() {
                    if sets1[keyindexvar.index as usize].count == 0 {
                        continue;
                    }
                    let hash = keyindexvar.hash;
                    let h0 = self.geth0(hash);
                    let h2 = self.geth2(hash);
                    keyindexvar.index += self.block_length;
                    stack.push(keyindexvar);

                    let mut s = unsafe { sets0.get_unchecked_mut(h0 as usize) };
                    s.xor_mask ^= hash;
                    s.count -= 1;
                    if s.count == 1 {
                        q0.push(KeyIndex {
                            index: h0,
                            hash: s.xor_mask,
                        })
                    }

                    let mut s = unsafe { sets2.get_unchecked_mut(h2 as usize) };
                    s.xor_mask ^= hash;
                    s.count -= 1;
                    if s.count == 1 {
                        q2.push(KeyIndex {
                            index: h2,
                            hash: s.xor_mask,
                        })
                    }
                }
                while let Some(mut keyindexvar) = q2.pop() {
                    if sets2[keyindexvar.index as usize].count == 0 {
                        continue;
                    }
                    let hash = keyindexvar.hash;
                    let h0 = self.geth0(hash);
                    let h1 = self.geth1(hash);
                    keyindexvar.index += 2 * self.block_length;
                    stack.push(keyindexvar);

                    let mut s = unsafe { sets0.get_unchecked_mut(h0 as usize) };
                    s.xor_mask ^= hash;
                    s.count -= 1;
                    if s.count == 1 {
                        q0.push(KeyIndex {
                            index: h0,
                            hash: s.xor_mask,
                        })
                    }
                    let mut s = unsafe { sets1.get_unchecked_mut(h1 as usize) };
                    s.xor_mask ^= hash;
                    s.count -= 1;
                    if s.count == 1 {
                        q1.push(KeyIndex {
                            index: h1,
                            hash: s.xor_mask,
                        })
                    }
                }
            }

            if stack.len() == size {
                break;
            }

            for item in sets0.iter_mut() {
                *item = XorSet::default();
            }
            for item in sets1.iter_mut() {
                *item = XorSet::default();
            }
            for item in sets2.iter_mut() {
                *item = XorSet::default();
            }
            self.seed = splitmix64(&mut rngcounter)
        }

        while let Some(ki) = stack.pop() {
            let mut val = fingerprint(ki.hash) as u8;
            if ki.index < self.block_length {
                let h1 = (self.geth1(ki.hash) + self.block_length) as usize;
                let h2 = (self.geth2(ki.hash) + 2 * self.block_length) as usize;
                val ^= self.finger_prints[h1] ^ self.finger_prints[h2];
            } else if ki.index < 2 * self.block_length {
                let h0 = self.geth0(ki.hash) as usize;
                let h2 = (self.geth2(ki.hash) + 2 * self.block_length) as usize;
                val ^= self.finger_prints[h0] ^ self.finger_prints[h2];
            } else {
                let h0 = self.geth0(ki.hash) as usize;
                let h1 = (self.geth1(ki.hash) + self.block_length) as usize;
                val ^= self.finger_prints[h0] ^ self.finger_prints[h1]
            }
            self.finger_prints[ki.index as usize] = val;
        }

        Ok(())
    }
}

impl<H> Xor8<H>
where H: BuildHasher
{
    /// Contains tell you whether the key is likely part of the set, with false
    /// positive rate.
    pub fn contains<K: ?Sized + Hash>(&self, key: &K) -> bool {
        let hashed_key = {
            let mut hasher = self.hash_builder.build_hasher();
            key.hash(&mut hasher);
            hasher.finish()
        };
        self.contains_key(hashed_key)
    }

    /// Contains tell you whether the key, as pre-computed digest form, is likely
    /// part of the set, with false positive rate.
    pub fn contains_key(&self, digest: u64) -> bool {
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

    #[allow(dead_code)]
    fn get_hasher(&self) -> H::Hasher {
        self.hash_builder.build_hasher()
    }
}

impl<H> Xor8<H>
where H: BuildHasher
{
    fn geth0h1h2(&self, k: u64) -> Hashes {
        let h = mixsplit(k, self.seed);
        Hashes {
            h,
            h0: reduce(h as u32, self.block_length),
            h1: reduce(h.rotate_left(21) as u32, self.block_length),
            h2: reduce(h.rotate_left(42) as u32, self.block_length),
        }
    }

    fn geth0(&self, hash: u64) -> u32 {
        let r0 = hash as u32;
        reduce(r0, self.block_length)
    }

    fn geth1(&self, hash: u64) -> u32 {
        let r1 = hash.rotate_left(21) as u32;
        reduce(r1, self.block_length)
    }

    fn geth2(&self, hash: u64) -> u32 {
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
    const SIGNATURE_V1: [u8; 4] = [b'^', b'T', b'L', 1];
    const SIGNATURE_V2: [u8; 4] = [b'^', b'T', b'L', 2];

    /// METADATA_LENGTH is size that required to write size of all the
    /// metadata of the serialized filter.
    // signature length + seed length + block-length +
    //      fingerprint length + hasher-builder length + fingerprint + hash-builder
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
        let finger_prints = buf[n..n + fp_len].to_vec();
        n += fp_len;
        // fetch the hash_builder
        let hash_builder: H = buf[n..n + hb_len].to_vec().into();

        Ok(Xor8 {
            keys: None,
            hash_builder,
            seed,
            block_length,
            finger_prints,
        })
    }

    fn from_bytes_v1(buf: Vec<u8>) -> io::Result<Self>
    where H: Default {
        use std::io::Error;

        // validate the buf first.
        if Self::METADATA_LENGTH > buf.len() {
            return Err(Error::new(ErrorKind::InvalidData, "invalid byte slice"));
        }
        if buf[..4] != Xor8::<H>::SIGNATURE_V1 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "File signature incorrect",
            ));
        }
        let fp_len = u32::from_be_bytes(buf[16..20].try_into().unwrap()) as usize;
        if buf[20..].len() < fp_len {
            return Err(Error::new(ErrorKind::InvalidData, "invalid byte slice"));
        }
        Ok(Xor8 {
            keys: None,
            hash_builder: H::default(),
            seed: u64::from_be_bytes(buf[4..12].try_into().unwrap()),
            block_length: u32::from_be_bytes(buf[12..16].try_into().unwrap()),
            finger_prints: buf[20..].to_vec(),
        })
    }
}
