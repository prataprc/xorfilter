//! Library implements xor-filter.
//!
//! This is a port of its
//! [original implementation](https://github.com/FastFilter/xorfilter)
//! written in golang.

fn murmur64(mut h: u64) -> u64 {
    h ^= h >> 33;
    h = h.overflowing_mul(0xff51afd7ed558ccd).0;
    h ^= h >> 33;
    h = h.overflowing_mul(0xc4ceb9fe1a85ec53).0;
    h ^= h >> 33;
    h
}

// returns random number, modifies the seed
fn splitmix64(seed: &mut u64) -> u64 {
    *seed = (*seed).overflowing_add(0x9E3779B97F4A7C15).0;
    let mut z = *seed;
    z = (z ^ (z >> 30)).overflowing_mul(0xBF58476D1CE4E5B9).0;
    z = (z ^ (z >> 27)).overflowing_mul(0x94D049BB133111EB).0;
    z ^ (z >> 31)
}

fn mixsplit(key: u64, seed: u64) -> u64 {
    murmur64(key.overflowing_add(seed).0)
}

fn rotl64(n: u64, c: i64) -> u64 {
    (n << (c & 63)) | (n >> ((-c) & 63))
}

fn reduce(hash: u32, n: u32) -> u32 {
    // http://lemire.me/blog/2016/06/27/a-fast-alternative-to-the-modulo-reduction/
    ((hash as u64) * (n as u64) >> 32) as u32
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

/// Type Xor8 is probabilistic data-structure to test membership of an
/// element in a set.
///
/// This implementation has a false positive rate of about 0.3%
/// and a memory usage of less than 9 bits per entry for sizeable sets.
pub struct Xor8 {
    seed: u64,
    block_length: u32,
    finger_prints: Vec<u8>,
}

impl Xor8 {
    /// Populate fills the filter with provided keys.
    ///
    /// The caller is responsible to ensure that there are no duplicate keys.
    pub fn populate(keys: &Vec<u64>) -> Box<Xor8> {
        let (size, mut rngcounter) = (keys.len(), 1_u64);
        let capacity = {
            let capacity = 32 + ((1.23 * (size as f64)).ceil() as u32);
            capacity / 3 * 3 // round it down to a multiple of 3
        };
        let mut filter: Box<Xor8> = {
            let mut filter = Box::new(Xor8 {
                seed: splitmix64(&mut rngcounter),
                block_length: capacity / 3,
                finger_prints: Vec::with_capacity(capacity as usize),
            });
            filter
                .finger_prints
                .resize(capacity as usize, Default::default());
            filter
        };

        let mut q0: Vec<KeyIndex> = Vec::with_capacity(filter.block_length as usize);
        q0.resize(filter.block_length as usize, Default::default());
        let mut q1: Vec<KeyIndex> = Vec::with_capacity(filter.block_length as usize);
        q1.resize(filter.block_length as usize, Default::default());
        let mut q2: Vec<KeyIndex> = Vec::with_capacity(filter.block_length as usize);
        q2.resize(filter.block_length as usize, Default::default());
        let mut stack: Vec<KeyIndex> = Vec::with_capacity(size);
        stack.resize(size, Default::default());
        let mut sets0: Vec<XorSet> = Vec::with_capacity(filter.block_length as usize);
        sets0.resize(filter.block_length as usize, Default::default());
        let mut sets1: Vec<XorSet> = Vec::with_capacity(filter.block_length as usize);
        sets1.resize(filter.block_length as usize, Default::default());
        let mut sets2: Vec<XorSet> = Vec::with_capacity(filter.block_length as usize);
        sets2.resize(filter.block_length as usize, Default::default());
        loop {
            for i in 0..size {
                let key = keys[i];
                let hs = filter.geth0h1h2(key);
                sets0[hs.h0 as usize].xor_mask ^= hs.h;
                sets0[hs.h0 as usize].count += 1;
                sets1[hs.h1 as usize].xor_mask ^= hs.h;
                sets1[hs.h1 as usize].count += 1;
                sets2[hs.h2 as usize].xor_mask ^= hs.h;
                sets2[hs.h2 as usize].count += 1;
            }
            // scan for values with a count of one
            let (mut q0_size, mut q1_size, mut q2_size) = (0, 0, 0);
            for i in 0..(filter.block_length as usize) {
                if sets0[i].count == 1 {
                    q0[q0_size].index = i as u32;
                    q0[q0_size].hash = sets0[i].xor_mask;
                    q0_size += 1;
                }
            }

            for i in 0..(filter.block_length as usize) {
                if sets1[i].count == 1 {
                    q1[q1_size].index = i as u32;
                    q1[q1_size].hash = sets1[i].xor_mask;
                    q1_size += 1;
                }
            }
            for i in 0..(filter.block_length as usize) {
                if sets2[i].count == 1 {
                    q2[q2_size].index = i as u32;
                    q2[q2_size].hash = sets2[i].xor_mask;
                    q2_size += 1;
                }
            }
            let mut stacksize = 0;
            while q0_size + q1_size + q2_size > 0 {
                while q0_size > 0 {
                    q0_size -= 1;
                    let (keyindexvar, index) = (q0[q0_size], q0[q0_size].index as usize);
                    if sets0[index].count == 0 {
                        // not actually possible after the initial scan.
                        continue;
                    }
                    let hash = keyindexvar.hash;
                    let h1 = filter.geth1(hash);
                    let h2 = filter.geth2(hash);
                    stack[stacksize] = keyindexvar;
                    stacksize += 1;
                    sets1[h1 as usize].xor_mask ^= hash;

                    sets1[h1 as usize].count -= 1;
                    if sets1[h1 as usize].count == 1 {
                        q1[q1_size].index = h1;
                        q1[q1_size].hash = sets1[h1 as usize].xor_mask;
                        q1_size += 1;
                    }
                    sets2[h2 as usize].xor_mask ^= hash;
                    sets2[h2 as usize].count -= 1;
                    if sets2[h2 as usize].count == 1 {
                        q2[q2_size].index = h2;
                        q2[q2_size].hash = sets2[h2 as usize].xor_mask;
                        q2_size += 1;
                    }
                }
                while q1_size > 0 {
                    q1_size -= 1;
                    let (mut keyindexvar, index) = (q1[q1_size], q1[q1_size].index as usize);
                    if sets1[index].count == 0 {
                        continue;
                    }
                    let hash = keyindexvar.hash;
                    let h0 = filter.geth0(hash);
                    let h2 = filter.geth2(hash);
                    keyindexvar.index += filter.block_length;
                    stack[stacksize] = keyindexvar;
                    stacksize += 1;
                    sets0[h0 as usize].xor_mask ^= hash;
                    sets0[h0 as usize].count -= 1;
                    if sets0[h0 as usize].count == 1 {
                        q0[q0_size].index = h0;
                        q0[q0_size].hash = sets0[h0 as usize].xor_mask;
                        q0_size += 1;
                    }
                    sets2[h2 as usize].xor_mask ^= hash;
                    sets2[h2 as usize].count -= 1;
                    if sets2[h2 as usize].count == 1 {
                        q2[q2_size].index = h2;
                        q2[q2_size].hash = sets2[h2 as usize].xor_mask;
                        q2_size += 1;
                    }
                }
                while q2_size > 0 {
                    q2_size -= 1;
                    let (mut keyindexvar, index) = (q2[q2_size], q2[q2_size].index as usize);
                    if sets2[index].count == 0 {
                        continue;
                    }
                    let hash = keyindexvar.hash;
                    let h0 = filter.geth0(hash);
                    let h1 = filter.geth1(hash);
                    keyindexvar.index += 2 * filter.block_length;

                    stack[stacksize] = keyindexvar;
                    stacksize += 1;
                    sets0[h0 as usize].xor_mask ^= hash;
                    sets0[h0 as usize].count -= 1;
                    if sets0[h0 as usize].count == 1 {
                        q0[q0_size].index = h0;
                        q0[q0_size].hash = sets0[h0 as usize].xor_mask;
                        q0_size += 1;
                    }
                    sets1[h1 as usize].xor_mask ^= hash;
                    sets1[h1 as usize].count -= 1;
                    if sets1[h1 as usize].count == 1 {
                        q1[q1_size].index = h1;
                        q1[q1_size].hash = sets1[h1 as usize].xor_mask;
                        q1_size += 1;
                    }
                }
            }

            if stacksize == size {
                break;
            }

            for i in 0..sets0.len() {
                sets0[i] = Default::default();
            }
            for i in 0..sets1.len() {
                sets1[i] = Default::default();
            }
            for i in 0..sets2.len() {
                sets2[i] = Default::default();
            }
            filter.seed = splitmix64(&mut rngcounter)
        }

        let mut stacksize = size;
        while stacksize > 0 {
            stacksize -= 1;
            let ki = stack[stacksize];
            let mut val = fingerprint(ki.hash) as u8;
            if ki.index < filter.block_length {
                val ^= filter.finger_prints[(filter.geth1(ki.hash) + filter.block_length) as usize]
                    ^ filter.finger_prints
                        [(filter.geth2(ki.hash) + 2 * filter.block_length) as usize]
            } else if ki.index < 2 * filter.block_length {
                val ^= filter.finger_prints[filter.geth0(ki.hash) as usize]
                    ^ filter.finger_prints
                        [(filter.geth2(ki.hash) + 2 * filter.block_length) as usize]
            } else {
                val ^= filter.finger_prints[filter.geth0(ki.hash) as usize]
                    ^ filter.finger_prints[(filter.geth1(ki.hash) + filter.block_length) as usize]
            }
            filter.finger_prints[ki.index as usize] = val;
        }
        filter
    }

    /// Contains tell you whether the key is likely part of the set.
    pub fn contains(&self, key: u64) -> bool {
        let hash = mixsplit(key, self.seed);
        let f = fingerprint(hash) as u8;
        let r0 = hash as u32;
        let r1 = rotl64(hash, 21) as u32;
        let r2 = rotl64(hash, 42) as u32;
        let h0 = reduce(r0, self.block_length) as usize;
        let h1 = (reduce(r1, self.block_length) + self.block_length) as usize;
        let h2 = (reduce(r2, self.block_length) + 2 * self.block_length) as usize;
        f == (self.finger_prints[h0] ^ self.finger_prints[h1] ^ self.finger_prints[h2])
    }

    fn geth0h1h2(&self, k: u64) -> Hashes {
        let mut answer: Hashes = Default::default();
        answer.h = mixsplit(k, self.seed);
        let r0 = answer.h as u32;
        let r1 = rotl64(answer.h, 21) as u32;
        let r2 = rotl64(answer.h, 42) as u32;

        answer.h0 = reduce(r0, self.block_length);
        answer.h1 = reduce(r1, self.block_length);
        answer.h2 = reduce(r2, self.block_length);
        return answer;
    }

    fn geth0(&self, hash: u64) -> u32 {
        let r0 = hash as u32;
        reduce(r0, self.block_length)
    }

    fn geth1(&self, hash: u64) -> u32 {
        let r1 = rotl64(hash, 21) as u32;
        reduce(r1, self.block_length)
    }

    fn geth2(&self, hash: u64) -> u32 {
        let r2 = rotl64(hash, 42) as u32;
        reduce(r2, self.block_length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{prelude::random, rngs::SmallRng, Rng, SeedableRng};

    #[test]
    fn test_basic() {
        let seed: u128 = random();
        println!("seed {}", seed);
        let mut rng = SmallRng::from_seed(seed.to_le_bytes());

        let testsize = 10000;
        let mut keys: Vec<u64> = Vec::with_capacity(testsize);
        keys.resize(testsize, Default::default());
        for i in 0..keys.len() {
            keys[i] = rng.gen();
        }

        let filter = Xor8::populate(&keys);
        for key in keys.into_iter() {
            assert!(filter.contains(key), "key {} not present", key);
        }

        let (falsesize, mut matches) = (1000000, 0_f64);
        let bpv = (filter.finger_prints.len() as f64) * 8.0 / (testsize as f64);
        println!("bits per entry {}", bpv);
        assert!(bpv < 10.0, "bpv({}) >= 10.0", bpv);

        for _ in 0..falsesize {
            if filter.contains(rng.gen()) {
                matches += 1_f64;
            }
        }
        let fpp = matches * 100.0 / (falsesize as f64);
        println!("false positive rate {}", fpp);
        assert!(fpp < 0.40, "fpp({}) >= 0.40", fpp);
    }
}
