use std::collections::HashSet;
use std::hash::BuildHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::Arc;

use crate::xor8::filter::fingerprint;
use crate::xor8::filter::splitmix64;
use crate::xor8::filter::XorSet;
use crate::xor8::Xor8;
use crate::BuildHasherDefault;

#[derive(Clone, Copy, Default)]
struct KeyIndex {
    hash: u64,
    index: u32,
}

/// Builds an Xor8 filter.
///
/// Example:
/// ```
/// # use xorfilter::xor8::Xor8Builder;
///
/// let mut b: Xor8Builder = Xor8Builder::new();
///
/// b.populate(&["foo", "bar"]);
/// let filter = b.build().unwrap();
///
/// assert!(filter.contains("foo"));
/// ```
#[derive(Clone, Debug)]
pub struct Xor8Builder<H = BuildHasherDefault>
where H: BuildHasher + Clone
{
    digests: HashSet<u64>,
    pub num_digests: usize,
    pub hash_builder: H,
}

impl<H> Default for Xor8Builder<H>
where H: BuildHasher + Clone + Default
{
    fn default() -> Self {
        Self {
            digests: Default::default(),
            num_digests: 0,
            hash_builder: H::default(),
        }
    }
}

impl<H> Xor8Builder<H>
where H: BuildHasher + Clone
{
    /// New Xor8 builder initialized with [BuildHasherDefault].
    pub fn new() -> Self
    where H: Default {
        Self::default()
    }

    /// New Xor8 builder initialized with supplied `hasher`.
    pub fn with_hasher(hash_builder: H) -> Self {
        Self {
            digests: HashSet::new(),
            num_digests: 0,
            hash_builder,
        }
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

    /// Insert 64-bit digest of a single key.
    ///
    /// Digest for the key shall be generated using the default-hasher or via hasher
    /// supplied via [Xor8Builder::with_hasher] method.
    pub fn insert<K: ?Sized + Hash>(&mut self, key: &K) {
        let digest = self.hash(key);

        self.digests.insert(digest);
        self.num_digests += 1;
    }

    /// Populate with 64-bit digests for a collection of keys of type `K`.
    ///
    /// Digest for key shall be generated using the default-hasher or via hasher supplied
    /// via [Xor8Builder::with_hasher] method.
    pub fn populate<'i, K: Hash + 'i, I: IntoIterator<Item = &'i K>>(&mut self, keys: I) {
        let mut n = 0;

        for key in keys.into_iter() {
            n += 1;

            let digest = self.hash(key);
            self.digests.insert(digest);
        }

        self.num_digests += n;
    }

    /// Populate with pre-compute collection of 64-bit digests.
    pub fn populate_digests<'i, I: IntoIterator<Item = &'i u64>>(&mut self, digests: I) {
        let mut n = 0;

        for digest in digests.into_iter() {
            n += 1;
            self.digests.insert(*digest);
        }

        self.num_digests += n;
    }

    /// Build bitmap for keys that where previously inserted using [Xor8Builder::insert],
    /// [Xor8Builder::populate] and [Xor8Builder::populate_digests] method.
    pub fn build(&mut self) -> Result<Xor8<H>, crate::Error> {
        let digests = self.digests.iter().copied().collect::<Vec<u64>>();
        self.build_from_digests(&digests)
    }

    /// Build a bitmap for pre-computed 64-bit digests for keys.
    ///
    /// If keys where previously inserted using [Xor8Builder::insert] or
    /// [Xor8Builder::populate] or [Xor8Builder::populate_digests] methods, they shall be
    /// ignored.
    ///
    /// It is upto the caller to ensure that digests are unique, that there no duplicates.
    pub fn build_from_digests(
        &mut self,
        digests: &[u64],
    ) -> Result<Xor8<H>, crate::Error> {
        let mut ff = Xor8::<H>::new(self.hash_builder.clone());

        ff.num_keys = Some(digests.len());
        let (size, mut rngcounter) = (digests.len(), 1_u64);
        let capacity = {
            let capacity = 32 + ((1.23 * (size as f64)).ceil() as u32);
            capacity / 3 * 3 // round it down to a multiple of 3
        };
        ff.seed = splitmix64(&mut rngcounter);
        ff.block_length = capacity / 3;
        ff.finger_prints = Arc::new(vec![u8::default(); capacity as usize]);

        let block_length = ff.block_length as usize;
        let mut q0: Vec<KeyIndex> = Vec::with_capacity(block_length);
        let mut q1: Vec<KeyIndex> = Vec::with_capacity(block_length);
        let mut q2: Vec<KeyIndex> = Vec::with_capacity(block_length);
        let mut stack: Vec<KeyIndex> = Vec::with_capacity(size);
        let mut sets0: Vec<XorSet> = vec![XorSet::default(); block_length];
        let mut sets1: Vec<XorSet> = vec![XorSet::default(); block_length];
        let mut sets2: Vec<XorSet> = vec![XorSet::default(); block_length];

        loop {
            for key in digests.iter() {
                let hs = ff.get_h0h1h2(*key);
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

            let iter = sets0.iter().enumerate().take(ff.block_length as usize);
            for (i, item) in iter {
                if item.count == 1 {
                    q0.push(KeyIndex {
                        index: i as u32,
                        hash: item.xor_mask,
                    });
                }
            }
            let iter = sets1.iter().enumerate().take(ff.block_length as usize);
            for (i, item) in iter {
                if item.count == 1 {
                    q1.push(KeyIndex {
                        index: i as u32,
                        hash: item.xor_mask,
                    });
                }
            }
            let iter = sets2.iter().enumerate().take(ff.block_length as usize);
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
                    let h1 = ff.get_h1(hash);
                    let h2 = ff.get_h2(hash);
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
                    let h0 = ff.get_h0(hash);
                    let h2 = ff.get_h2(hash);
                    keyindexvar.index += ff.block_length;
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
                    let h0 = ff.get_h0(hash);
                    let h1 = ff.get_h1(hash);
                    keyindexvar.index += 2 * ff.block_length;
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
            ff.seed = splitmix64(&mut rngcounter)
        }

        while let Some(ki) = stack.pop() {
            let mut val = fingerprint(ki.hash) as u8;
            if ki.index < ff.block_length {
                let h1 = (ff.get_h1(ki.hash) + ff.block_length) as usize;
                let h2 = (ff.get_h2(ki.hash) + 2 * ff.block_length) as usize;
                val ^= ff.finger_prints[h1] ^ ff.finger_prints[h2];
            } else if ki.index < 2 * ff.block_length {
                let h0 = ff.get_h0(ki.hash) as usize;
                let h2 = (ff.get_h2(ki.hash) + 2 * ff.block_length) as usize;
                val ^= ff.finger_prints[h0] ^ ff.finger_prints[h2];
            } else {
                let h0 = ff.get_h0(ki.hash) as usize;
                let h1 = (ff.get_h1(ki.hash) + ff.block_length) as usize;
                val ^= ff.finger_prints[h0] ^ ff.finger_prints[h1]
            }
            Arc::get_mut(&mut ff.finger_prints).unwrap()[ki.index as usize] = val;
        }

        Ok(ff)
    }
}
