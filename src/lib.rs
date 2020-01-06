//! Library implements xor-filter.
//!
//! This is a port of its
//! [original implementation](https://github.com/FastFilter/xorfilter)
//! written in golang.

use std::fs::File;
use std::io;
use std::io::{Error, ErrorKind, Read, Write};

fn murmur64(mut h: u64) -> u64 {
    h ^= h >> 33;
    h = h.wrapping_mul(0xff51afd7ed558ccd);
    h ^= h >> 33;
    h = h.wrapping_mul(0xc4ceb9fe1a85ec53);
    h ^= h >> 33;
    h
}

// returns random number, modifies the seed
fn splitmix64(seed: &mut u64) -> u64 {
    *seed = (*seed).wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *seed;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

fn mixsplit(key: u64, seed: u64) -> u64 {
    murmur64(key.wrapping_add(seed))
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
#[derive(PartialEq, Debug)]
pub struct Xor8 {
    seed: u64,
    block_length: u32,
    finger_prints: Vec<u8>,
}

impl Xor8 {
    /// Populate fills the filter with provided keys.
    ///
    /// The caller is responsible to ensure that there are no duplicate keys.
    pub fn new(keys: &Vec<u64>) -> Self {
        let (size, mut rngcounter) = (keys.len(), 1_u64);
        let capacity = {
            let capacity = 32 + ((1.23 * (size as f64)).ceil() as u32);
            capacity / 3 * 3 // round it down to a multiple of 3
        };
        let mut filter: Xor8 = Xor8 {
            seed: splitmix64(&mut rngcounter),
            block_length: capacity / 3,
            finger_prints: vec![Default::default(); capacity as usize],
        };

        let block_length = filter.block_length as usize;
        let mut q0: Vec<KeyIndex> = Vec::with_capacity(block_length);
        let mut q1: Vec<KeyIndex> = Vec::with_capacity(block_length);
        let mut q2: Vec<KeyIndex> = Vec::with_capacity(block_length);
        let mut stack: Vec<KeyIndex> = Vec::with_capacity(size);
        let mut sets0: Vec<XorSet> = vec![Default::default(); block_length];
        let mut sets1: Vec<XorSet> = vec![Default::default(); block_length];
        let mut sets2: Vec<XorSet> = vec![Default::default(); block_length];
        loop {
            for key in keys {
                let hs = filter.geth0h1h2(*key);
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

            for i in 0..(filter.block_length as usize) {
                if sets0[i].count == 1 {
                    q0.push(KeyIndex{index: i as u32, hash: sets0[i].xor_mask});
                }
            }

            for i in 0..(filter.block_length as usize) {
                if sets1[i].count == 1 {
                    q1.push(KeyIndex{index: i as u32, hash: sets1[i].xor_mask});
                }
            }
            for i in 0..(filter.block_length as usize) {
                if sets2[i].count == 1 {
                    q2.push(KeyIndex{index: i as u32, hash: sets2[i].xor_mask});
                }
            }

            stack.clear();

            while !q0.is_empty() || !q1.is_empty() || !q2.is_empty() {
                while let Some(keyindexvar) = q0.pop(){
                    if sets0[keyindexvar.index as usize].count == 0 {
                        // not actually possible after the initial scan.
                        continue;
                    }
                    let hash = keyindexvar.hash;
                    let h1 = filter.geth1(hash);
                    let h2 = filter.geth2(hash);
                    stack.push(keyindexvar);

                    sets1[h1 as usize].xor_mask ^= hash;
                    sets1[h1 as usize].count -= 1;
                    if sets1[h1 as usize].count == 1 {
                        q1.push(KeyIndex{index: h1, hash: sets1[h1 as usize].xor_mask})
                    }
                    sets2[h2 as usize].xor_mask ^= hash;
                    sets2[h2 as usize].count -= 1;
                    if sets2[h2 as usize].count == 1 {
                        q2.push(KeyIndex{index: h2, hash: sets2[h2 as usize].xor_mask})
                    }
                }
                while let Some(mut keyindexvar) = q1.pop() {
                    if sets1[keyindexvar.index as usize].count == 0 {
                        continue;
                    }
                    let hash = keyindexvar.hash;
                    let h0 = filter.geth0(hash);
                    let h2 = filter.geth2(hash);
                    keyindexvar.index += filter.block_length;
                    stack.push(keyindexvar);

                    sets0[h0 as usize].xor_mask ^= hash;
                    sets0[h0 as usize].count -= 1;
                    if sets0[h0 as usize].count == 1 {
                        q0.push(KeyIndex{index: h0, hash: sets0[h0 as usize].xor_mask})
                    }
                    sets2[h2 as usize].xor_mask ^= hash;
                    sets2[h2 as usize].count -= 1;
                    if sets2[h2 as usize].count == 1 {
                        q2.push(KeyIndex{index: h2, hash: sets2[h2 as usize].xor_mask})
                    }
                }
                while let Some(mut keyindexvar) = q2.pop() {
                    if sets2[keyindexvar.index as usize].count == 0 {
                        continue;
                    }
                    let hash = keyindexvar.hash;
                    let h0 = filter.geth0(hash);
                    let h1 = filter.geth1(hash);
                    keyindexvar.index += 2 * filter.block_length;
                    stack.push(keyindexvar);

                    sets0[h0 as usize].xor_mask ^= hash;
                    sets0[h0 as usize].count -= 1;
                    if sets0[h0 as usize].count == 1 {
                        q0.push(KeyIndex{index: h0, hash: sets0[h0 as usize].xor_mask})
                    }
                    sets1[h1 as usize].xor_mask ^= hash;
                    sets1[h1 as usize].count -= 1;
                    if sets1[h1 as usize].count == 1 {
                        q1.push(KeyIndex{index: h1, hash: sets1[h1 as usize].xor_mask})
                    }
                }
            }

            if stack.len() == size {
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

        while let Some(ki) = stack.pop() {
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
        let r1 = hash.rotate_left(21) as u32;
        let r2 = hash.rotate_left(42) as u32;
        let h0 = reduce(r0, self.block_length) as usize;
        let h1 = (reduce(r1, self.block_length) + self.block_length) as usize;
        let h2 = (reduce(r2, self.block_length) + 2 * self.block_length) as usize;
        f == (self.finger_prints[h0] ^ self.finger_prints[h1] ^ self.finger_prints[h2])
    }

    fn geth0h1h2(&self, k: u64) -> Hashes {
        let mut answer: Hashes = Default::default();
        answer.h = mixsplit(k, self.seed);
        let r0 = answer.h as u32;
        let r1 = answer.h.rotate_left(21) as u32;
        let r2 = answer.h.rotate_left(42) as u32;

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
        let r1 = hash.rotate_left(21) as u32;
        reduce(r1, self.block_length)
    }

    fn geth2(&self, hash: u64) -> u32 {
        let r2 = hash.rotate_left(42) as u32;
        reduce(r2, self.block_length)
    }
}

impl Xor8 {
    /// File signature write on first 4 bytes of file.
    /// ^ stands for xor
    /// TL stands for filter
    /// 1 stands for version 1
    const SIGNATURE_V1: [u8; 4] = [b'^', b'T', b'L', 1];

    /// Write to file in binary format
    /// TODO Add chechsum of finger_prints into file headers
    pub fn write_file(&self, path: &str) -> io::Result<usize> {
        let n_fp = self.finger_prints.len() as u32; // u32 should be enough (4GB finger_prints)

        let mut f = File::create(path)?;
        let mut n_write = 0;
        n_write += f.write(&Xor8::SIGNATURE_V1)?; // 4 bytes
        n_write += f.write(&self.seed.to_be_bytes())?; // 8 bytes
        n_write += f.write(&self.block_length.to_be_bytes())?; // 4 bytes
        n_write += f.write(&n_fp.to_be_bytes())?; // 4 bytes
        n_write += f.write(&self.finger_prints)?;

        let n_expect = 4 + 8 + 4 + 4 + self.finger_prints.len();
        if n_write == n_expect {
            Ok(n_write)
        } else {
            Err(Error::new(
                ErrorKind::InvalidData,
                "Write data size mismatch",
            ))
        }
    }

    /// Read from file in binary format
    pub fn read_file(path: &str) -> io::Result<Self> {
        let mut buf_signature = [0_u8; 4];
        let mut buf_seed = [0_u8; 8];
        let mut buf_block_length = [0_u8; 4];
        let mut buf_n_fp = [0_u8; 4];

        let mut f = File::open(path)?;
        f.read_exact(&mut buf_signature)?;
        if buf_signature
            .iter()
            .zip(&Xor8::SIGNATURE_V1)
            .any(|(a, b)| a != b)
        {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "File signature incorrect",
            ));
        }

        f.read_exact(&mut buf_seed)?;
        f.read_exact(&mut buf_block_length)?;
        f.read_exact(&mut buf_n_fp)?;
        let n_fp = u32::from_be_bytes(buf_n_fp) as usize;
        let mut finger_prints: Vec<u8> = Vec::with_capacity(n_fp);
        let n_read = f.read_to_end(&mut finger_prints)?;

        if n_read == n_fp {
            Ok(Xor8 {
                seed: u64::from_be_bytes(buf_seed),
                block_length: u32::from_be_bytes(buf_block_length),
                finger_prints,
            })
        } else {
            Err(Error::new(
                ErrorKind::InvalidData,
                "Read data size mismatch",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{prelude::random, rngs::SmallRng, Rng, SeedableRng};

    #[test]
    fn test_basic1() {
        let seed: u128 = random();
        println!("seed {}", seed);
        let mut rng = SmallRng::from_seed(seed.to_le_bytes());

        let testsize = 10000;
        let mut keys: Vec<u64> = Vec::with_capacity(testsize);
        keys.resize(testsize, Default::default());
        for i in 0..keys.len() {
            keys[i] = rng.gen();
        }

        let filter = Xor8::new(&keys);
        for key in keys.into_iter() {
            assert!(filter.contains(key), "key {} not present", key);
        }

        let (falsesize, mut matches) = (1000000, 0_f64);
        let bpv = (filter.finger_prints.len() as f64) * 8.0 / (testsize as f64);
        println!("bits per entry {} bits", bpv);
        assert!(bpv < 10.0, "bpv({}) >= 10.0", bpv);

        for _ in 0..falsesize {
            if filter.contains(rng.gen()) {
                matches += 1_f64;
            }
        }
        let fpp = matches * 100.0 / (falsesize as f64);
        println!("false positive rate {}%", fpp);
        assert!(fpp < 0.40, "fpp({}) >= 0.40", fpp);
    }

    #[test]
    fn test_basic2() {
        let mut seed: u64 = random();
        println!("seed {}", seed);

        let testsize = 10000;
        let mut keys: Vec<u64> = Vec::with_capacity(testsize);
        keys.resize(testsize, Default::default());
        for i in 0..keys.len() {
            keys[i] = splitmix64(&mut seed);
        }

        let filter = Xor8::new(&keys);
        for key in keys.into_iter() {
            assert!(filter.contains(key), "key {} not present", key);
        }

        let (falsesize, mut matches) = (1000000, 0_f64);
        let bpv = (filter.finger_prints.len() as f64) * 8.0 / (testsize as f64);
        println!("bits per entry {} bits", bpv);
        assert!(bpv < 10.0, "bpv({}) >= 10.0", bpv);

        for _ in 0..falsesize {
            let v = splitmix64(&mut seed);
            if filter.contains(v) {
                matches += 1_f64;
            }
        }
        let fpp = matches * 100.0 / (falsesize as f64);
        println!("false positive rate {}%", fpp);
        assert!(fpp < 0.40, "fpp({}) >= 0.40", fpp);
    }
}
