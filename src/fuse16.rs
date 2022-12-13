#[allow(unused_imports)]
use std::collections::hash_map::DefaultHasher;
#[allow(unused_imports)]
use std::collections::hash_map::RandomState;
use std::collections::BTreeMap;
use std::hash::BuildHasher;
use std::hash::Hash;
use std::hash::Hasher;
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

use crate::fuse8::BinaryHashes;
use crate::BuildHasherDefault;
use crate::Error;
use crate::Result;

// probabillity of success should always be > 0.5 so 100 iterations is highly unlikely.
const XOR_MAX_ITERATIONS: usize = 100;

#[inline]
pub fn binary_fuse16_fingerprint(hash: u64) -> u64 {
    hash ^ (hash >> 32)
}

/// Type Fuse16 is probabilistic data-structure to test membership of an element in a set.
///
/// Fuse16 is parametrized over type `H` which is expected to implement [BuildHasher]
/// trait, like types [RandomState] and [BuildHasherDefault]. When not supplied,
/// [BuildHasherDefault] is used as the default hash-builder.
///
/// If `RandomState` is used as BuildHasher, `std` has got this to say
/// > _A particular instance RandomState will create the same instances
/// > of Hasher, but the hashers created by two different RandomState
/// > instances are unlikely to produce the same result for the same values._
///
/// If [DefaultHasher] is used as BuildHasher, `std` has got this to say,
/// > _The internal algorithm is not specified, and so its hashes
/// > should not be relied upon over releases._
///
/// The default type for parameter `H` might change when a reliable and commonly used
/// BuildHasher type available.
pub struct Fuse16<H = BuildHasherDefault>
where H: BuildHasher
{
    keys: Option<BTreeMap<u64, ()>>,
    pub hash_builder: H,
    pub seed: u64,
    pub num_keys: Option<usize>,
    pub segment_length: u32,
    pub segment_length_mask: u32,
    pub segment_count: u32,
    pub segment_count_length: u32,
    pub finger_prints: Arc<Vec<u16>>,
}

impl<H> Clone for Fuse16<H>
where H: Clone + BuildHasher
{
    fn clone(&self) -> Self {
        Fuse16 {
            keys: Some(BTreeMap::new()),
            hash_builder: self.hash_builder.clone(),
            seed: self.seed,
            num_keys: self.num_keys,
            segment_length: self.segment_length,
            segment_length_mask: self.segment_length_mask,
            segment_count: self.segment_count,
            segment_count_length: self.segment_count_length,
            finger_prints: Arc::clone(&self.finger_prints),
        }
    }
}

impl<H> Fuse16<H>
where H: BuildHasher
{
    #[inline]
    fn binary_fuse16_hash_batch(&self, hash: u64) -> BinaryHashes {
        use crate::fuse8::binary_fuse_mulhi;

        let mut ans = BinaryHashes::default();

        ans.h0 = binary_fuse_mulhi(hash, self.segment_count_length.into()) as u32;
        ans.h1 = ans.h0 + self.segment_length;
        ans.h2 = ans.h1 + self.segment_length;
        ans.h1 ^= ((hash >> 18) as u32) & self.segment_length_mask;
        ans.h2 ^= (hash as u32) & self.segment_length_mask;
        ans
    }

    #[inline]
    fn binary_fuse16_hash(&self, index: u32, hash: u64) -> u32 {
        use crate::fuse8::binary_fuse_mulhi;

        let mut h = binary_fuse_mulhi(hash, self.segment_count_length.into());
        h += (index * self.segment_length) as u64;
        // keep the lower 36 bits
        let hh = hash & ((1_u64 << 36) - 1);
        // index 0: right shift by 36; index 1: right shift by 18; index 2: no shift
        h ^= (hh >> (36 - 18 * index)) & (self.segment_length_mask as u64);

        h as u32
    }
}

impl<H> Fuse16<H>
where H: BuildHasher
{
    /// New Fuse16 instance that can index size number of keys. Internal data-structures
    /// are pre-allocated for `size`.  `size` should be at least 2.
    pub fn new(size: u32) -> Fuse16<H>
    where H: Default {
        Self::with_hasher(size, H::default())
    }

    /// New Fuse16 instance initialized with supplied hasher.
    pub fn with_hasher(size: u32, hash_builder: H) -> Fuse16<H> {
        use std::cmp;

        use crate::fuse8::binary_fuse_calculate_segment_length;
        use crate::fuse8::binary_fuse_calculate_size_factor;

        let arity = 3_u32;

        let segment_length = match size {
            0 => 4,
            size => cmp::min(binary_fuse_calculate_segment_length(arity, size), 262144),
        };

        let segment_length_mask = segment_length - 1;
        let mut array_length = {
            let size_factor = binary_fuse_calculate_size_factor(arity, size);
            let cap = match size {
                0 | 1 => 0,
                size => ((size as f64) * size_factor).round() as u32,
            };
            let n = ((cap + segment_length - 1) / segment_length).wrapping_sub(arity - 1);
            (n.wrapping_add(arity) - 1) * segment_length
        };

        let mut segment_count = (array_length + segment_length - 1) / segment_length;
        segment_count = if segment_count <= (arity - 1) {
            1
        } else {
            segment_count - (arity - 1)
        };

        array_length = (segment_count + arity - 1) * segment_length;
        let segment_count_length = segment_count * segment_length;

        Fuse16 {
            keys: Some(BTreeMap::new()),
            hash_builder,
            seed: u64::default(),
            num_keys: None,
            segment_length,
            segment_length_mask,
            segment_count,
            segment_count_length,
            finger_prints: Arc::new(vec![0; array_length as usize]),
        }
    }
}

impl<H> Fuse16<H>
where H: BuildHasher
{
    /// Return the size of index.
    #[inline]
    pub fn size_of(&self) -> usize {
        std::mem::size_of::<Self>() + (self.finger_prints.len() * 2)
    }

    /// Insert 64-bit digest of a single key. Digest for the key shall be generated
    /// using the default-hasher or via hasher supplied via [Fuse16::with_hasher] method.
    pub fn insert<K: ?Sized + Hash>(&mut self, key: &K) {
        let digest = {
            let mut hasher = self.hash_builder.build_hasher();
            key.hash(&mut hasher);
            hasher.finish()
        };
        if let Some(x) = self.num_keys.as_mut() {
            *x += 1
        }
        self.keys.as_mut().unwrap().insert(digest, ());
    }

    /// Populate with 64-bit digests for a collection of keys of type `K`. Digest for
    /// key shall be generated using the default-hasher or via hasher supplied
    /// via [Fuse16::with_hasher] method.
    pub fn populate<K: Hash>(&mut self, keys: &[K]) {
        if let Some(x) = self.num_keys.as_mut() {
            *x += keys.len()
        }

        keys.iter().for_each(|key| {
            let mut hasher = self.hash_builder.build_hasher();
            key.hash(&mut hasher);
            self.keys.as_mut().unwrap().insert(hasher.finish(), ());
        })
    }

    /// Populate with pre-compute collection of 64-bit digests.
    pub fn populate_keys(&mut self, digests: &[u64]) {
        if let Some(x) = self.num_keys.as_mut() {
            *x += digests.len()
        }

        for digest in digests.iter() {
            self.keys.as_mut().unwrap().insert(*digest, ());
        }
    }
    // construct the filter, returns true on success, false on failure.
    // most likely, a failure is due to too high a memory usage
    // size is the number of keys
    // The caller is responsable for calling binary_fuse16_allocate(size,filter)
    // before. The caller is responsible to ensure that there are no duplicated
    // keys. The inner loop will run up to XOR_MAX_ITERATIONS times (default on
    // 100), it should never fail, except if there are duplicated keys. If it fails,
    // a return value of false is provided.
    /// Build bitmap for keys that where previously inserted using [Fuse16::insert],
    /// [Fuse16::populate] and [Fuse16::populate_keys] method.
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
    /// previously inserted using [Fuse16::insert] or [Fuse16::populate] or
    /// [Fuse16::populate_keys] methods, they shall be ignored.
    ///
    /// It is upto the caller to ensure that digests are unique, that there no
    /// duplicates.
    pub fn build_keys(&mut self, digests: &[u64]) -> Result<()> {
        use crate::fuse8::binary_fuse_mod3;
        use crate::fuse8::binary_fuse_murmur64;
        use crate::fuse8::binary_fuse_rng_splitmix64;

        let mut rng_counter = 0x726b2b9d438b9d4d_u64;
        let capacity = self.finger_prints.len();
        let size = digests.len();

        self.num_keys = Some(digests.len());
        self.seed = binary_fuse_rng_splitmix64(&mut rng_counter);
        let mut reverse_order: Vec<u64> = vec![0; size + 1];
        let mut reverse_h: Vec<u8> = vec![0; size];
        let mut alone: Vec<u32> = vec![0; capacity];
        let mut t2count: Vec<u8> = vec![0; capacity];
        let mut t2hash: Vec<u64> = vec![0; capacity];

        let mut block_bits: u32 = 1;
        while (1_u32 << block_bits) < self.segment_count {
            block_bits += 1;
        }
        let block = 1_u32 << block_bits;

        let mut start_pos: Vec<u32> = vec![0; 1 << block_bits];

        let mut h012 = [0_u32; 5];

        reverse_order[size] = 1; // sentinel
        let mut iter = 0..=XOR_MAX_ITERATIONS;
        loop {
            if iter.next().is_none() {
                err_at!(Fatal, msg: "Too many iterations. Are all your keys unique?")?;
            }

            for i in 0_u32..block {
                // important : i * size would overflow as a 32-bit number in some
                // cases.
                start_pos[i as usize] =
                    (((i as u64) * (size as u64)) >> block_bits) as u32;
            }

            let mask_block = (block - 1) as u64;
            for (_, digest) in digests.iter().enumerate().take(size) {
                let hash: u64 = binary_fuse_murmur64(digest.wrapping_add(self.seed));
                let mut segment_index: u64 = hash >> (64 - block_bits);
                while reverse_order[start_pos[segment_index as usize] as usize] != 0 {
                    segment_index += 1;
                    segment_index &= mask_block;
                }
                reverse_order[start_pos[segment_index as usize] as usize] = hash;
                start_pos[segment_index as usize] += 1;
            }

            let mut error: isize = 0;
            for (_, rev_order) in reverse_order.iter().enumerate().take(size) {
                let hash: u64 = *rev_order;

                let h0: usize = self.binary_fuse16_hash(0, hash) as usize;
                t2count[h0] = t2count[h0].wrapping_add(4);
                t2hash[h0] ^= hash;

                let h1: usize = self.binary_fuse16_hash(1, hash) as usize;
                t2count[h1] = t2count[h1].wrapping_add(4);
                t2count[h1] ^= 1;
                t2hash[h1] ^= hash;

                let h2: usize = self.binary_fuse16_hash(2, hash) as usize;
                t2count[h2] = t2count[h2].wrapping_add(4);
                t2hash[h2] ^= hash;
                t2count[h2] ^= 2;

                error = if t2count[h0] < 4 { 1 } else { error };
                error = if t2count[h1] < 4 { 1 } else { error };
                error = if t2count[h2] < 4 { 1 } else { error };
            }

            if error > 0 {
                reverse_order.fill(0);
                reverse_order[size] = 1; // sentinel
                t2count.fill(0);
                t2hash.fill(0);
                self.seed = binary_fuse_rng_splitmix64(&mut rng_counter);
                continue;
            }

            let mut q_size = 0_usize; // End of key addition

            // Add sets with one key to the queue.
            for (i, x) in t2count.iter().enumerate().take(capacity) {
                alone[q_size] = i as u32;
                q_size += if (x >> 2) == 1 { 1 } else { 0 };
            }

            let mut stack_size = 0_usize;

            while q_size > 0 {
                q_size -= 1;
                let index = alone[q_size] as usize;
                if (t2count[index] >> 2) == 1 {
                    let hash: u64 = t2hash[index];

                    //h012[0] = binary_fuse16_hash(0, hash, self);
                    h012[1] = self.binary_fuse16_hash(1, hash);
                    h012[2] = self.binary_fuse16_hash(2, hash);
                    h012[3] = self.binary_fuse16_hash(0, hash); // == h012[0];
                    h012[4] = h012[1];

                    let found: u8 = t2count[index] & 3;
                    reverse_h[stack_size] = found;
                    reverse_order[stack_size] = hash;
                    stack_size += 1;

                    let other_index1: u32 = h012[(found + 1) as usize];
                    alone[q_size] = other_index1;
                    q_size += if (t2count[other_index1 as usize] >> 2) == 2 {
                        1
                    } else {
                        0
                    };

                    t2count[other_index1 as usize] -= 4;
                    t2count[other_index1 as usize] ^= binary_fuse_mod3(found + 1);
                    t2hash[other_index1 as usize] ^= hash;

                    let other_index2: u32 = h012[(found + 2) as usize];
                    alone[q_size] = other_index2;
                    q_size += if (t2count[other_index2 as usize] >> 2) == 2 {
                        1
                    } else {
                        0
                    };
                    t2count[other_index2 as usize] -= 4;
                    t2count[other_index2 as usize] ^= binary_fuse_mod3(found + 2);
                    t2hash[other_index2 as usize] ^= hash;
                }
            }

            if stack_size == size {
                break; // success
            }

            reverse_order.fill(0);
            reverse_order[size] = 1; // sentinel
            t2count.fill(0);
            t2hash.fill(0);

            self.seed = binary_fuse_rng_splitmix64(&mut rng_counter);
        }

        for i in (0_usize..size).rev() {
            // the hash of the key we insert next
            let hash: u64 = reverse_order[i];
            let xor2: u16 = binary_fuse16_fingerprint(hash) as u16;
            let found: usize = reverse_h[i] as usize;
            h012[0] = self.binary_fuse16_hash(0, hash);
            h012[1] = self.binary_fuse16_hash(1, hash);
            h012[2] = self.binary_fuse16_hash(2, hash);
            h012[3] = h012[0];
            h012[4] = h012[1];

            Arc::get_mut(&mut self.finger_prints).unwrap()[h012[found] as usize] = xor2
                ^ self.finger_prints[h012[found + 1] as usize]
                ^ self.finger_prints[h012[found + 2] as usize];
        }

        Ok(())
    }
}

impl<H> Fuse16<H>
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
        let digest = {
            let mut hasher = self.hash_builder.build_hasher();
            key.hash(&mut hasher);
            hasher.finish()
        };
        self.contains_key(digest)
    }

    /// Contains tell you whether the key, as pre-computed digest form, is likely
    /// part of the set, with false positive rate.
    pub fn contains_key(&self, digest: u64) -> bool {
        use crate::fuse8::binary_fuse_mix_split;

        let hash = binary_fuse_mix_split(digest, self.seed);
        let mut f = binary_fuse16_fingerprint(hash) as u16;
        let BinaryHashes { h0, h1, h2 } = self.binary_fuse16_hash_batch(hash);
        f ^= self.finger_prints[h0 as usize]
            ^ self.finger_prints[h1 as usize]
            ^ self.finger_prints[h2 as usize];
        f == 0
    }

    #[allow(dead_code)]
    fn get_hasher(&self) -> H::Hasher {
        self.hash_builder.build_hasher()
    }
}

//------ Implement cbordata related functionalities

// Intermediate type to serialize and de-serialized Fuse16 into bytes.
#[cfg(feature = "cbordata")]
#[derive(Cborize)]
struct CborFuse16 {
    hash_builder: Vec<u8>,
    seed: u64,
    num_keys: Option<usize>,
    segment_length: u32,
    segment_length_mask: u32,
    segment_count: u32,
    segment_count_length: u32,
    finger_prints: Vec<u16>,
}

#[cfg(feature = "cbordata")]
impl CborFuse16 {
    const ID: &'static str = "fuse8/0.0.1";
}

#[cfg(feature = "cbordata")]
impl<H> IntoCbor for Fuse16<H>
where H: BuildHasher + Into<Vec<u8>>
{
    fn into_cbor(self) -> cbor::Result<Cbor> {
        let val = CborFuse16 {
            hash_builder: self.hash_builder.into(),
            seed: self.seed,
            num_keys: self.num_keys,
            segment_length: self.segment_length,
            segment_length_mask: self.segment_length_mask,
            segment_count: self.segment_count,
            segment_count_length: self.segment_count_length,
            finger_prints: self.finger_prints.to_vec(),
        };
        val.into_cbor()
    }
}

#[cfg(feature = "cbordata")]
impl<H> FromCbor for Fuse16<H>
where H: BuildHasher + From<Vec<u8>>
{
    fn from_cbor(val: Cbor) -> cbor::Result<Self> {
        let val = CborFuse16::from_cbor(val)?;

        let filter = Fuse16 {
            keys: None,
            hash_builder: val.hash_builder.into(),
            seed: val.seed,
            num_keys: val.num_keys,
            segment_length: val.segment_length,
            segment_length_mask: val.segment_length_mask,
            segment_count: val.segment_count,
            segment_count_length: val.segment_count_length,
            finger_prints: Arc::new(val.finger_prints),
        };

        Ok(filter)
    }
}

#[cfg(test)]
#[path = "fuse16_test.rs"]
mod fuse16_test;
