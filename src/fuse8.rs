// probabillity of success should always be > 0.5 so 100 iterations is highly unlikely.
const XOR_MAX_ITERATIONS: usize = 100;

macro_rules! alloc_locals {
    ($size:ident, $cap:ident) => {{
        let mut reverse_order: Vec<u64> = Vec::with_capacity($size + 1);
        reverse_order.resize($size + 1, u64::default());

        let mut reverse_h: Vec<u8> = Vec::with_capacity($size);
        reverse_h.resize($size, 0);

        let mut alone: Vec<u32> = Vec::with_capacity($cap);
        alone.resize($cap, 0);

        let mut t2count: Vec<u8> = Vec::with_capacity($cap);
        t2count.resize($cap, 0);

        let mut t2hash: Vec<u64> = Vec::with_capacity($cap);
        t2hash.resize($cap, 0);

        (reverse_order, reverse_h, alone, t2count, t2hash)
    }};
}

#[inline]
fn binary_fuse_murmur64(mut h: u64) -> u64 {
    h ^= h >> 33;
    h *= 0xff51afd7ed558ccd_u64;
    h ^= h >> 33;
    h *= 0xc4ceb9fe1a85ec53_u64;
    h ^= h >> 33;
    h
}

#[inline]
fn binary_fuse_mix_split(key: u64, seed: u64) -> u64 {
    binary_fuse_murmur64(key + seed)
}

#[allow(dead_code)]
#[inline]
fn binary_fuse_rotl64(n: u64, c: u32) -> u64 {
    n.rotate_left(c)
}

#[allow(dead_code)]
#[inline]
fn binary_fuse_reduce(hash: u32, n: u32) -> u32 {
    // http://lemire.me/blog/2016/06/27/a-fast-alternative-to-the-modulo-reduction/
    (((hash as u64) * (n as u64)) >> 32) as u32
}

#[inline]
fn binary_fuse8_fingerprint(hash: u64) -> u64 {
    return hash ^ (hash >> 32);
}

// returns random number, modifies the seed
fn binary_fuse_rng_splitmix64(seed: &mut u64) -> u64 {
    *seed += 0x9E3779B97F4A7C15_u64;
    let mut z = *seed;
    z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9_u64;
    z = (z ^ (z >> 27)) * 0x94D049BB133111EB_u64;
    z ^ (z >> 31)
}

#[inline]
fn binary_fuse_mulhi(a: u64, b: u64) -> u64 {
    (((a as u128) * (b as u128)) >> 64) as u64
}

#[inline]
fn binary_fuse_calculate_segment_length(arity: u32, size: u32) -> u32 {
    let ln_size = (size as f64).ln();

    // These parameters are very sensitive. Replacing 'floor' by 'round' can
    // substantially affect the construction time.
    match arity {
        3 => 1_u32 << ((ln_size / 3.33_f64.ln() + 2.25).floor() as u32),
        4 => 1_u32 << ((ln_size / 2.91_f64.ln() - 0.50).floor() as u32),
        _ => 65536,
    }
}

#[inline]
fn binary_fuse8_max(a: f64, b: f64) -> f64 {
    if a < b {
        b
    } else {
        a
    }
}

#[inline]
fn binary_fuse_calculate_size_factor(arity: u32, size: u32) -> f64 {
    let ln_size = (size as f64).ln();
    match arity {
        3 => binary_fuse8_max(1.125, 0.875 + 0.250 * 1000000.0_f64.ln() / ln_size),
        4 => binary_fuse8_max(1.075, 0.770 + 0.305 * 0600000.0_f64.ln() / ln_size),
        _ => 2.0,
    }
}

#[inline]
fn binary_fuse_mod3(x: u8) -> u8 {
    if x > 2 {
        x - 3
    } else {
        x
    }
}

pub struct Fuse8 {
    seed: u64,
    segment_length: u32,
    segment_length_mask: u32,
    segment_count: u32,
    segment_count_length: u32,
    finger_prints: Vec<u8>,
}

#[derive(Default)]
struct BinaryHashes {
    h0: u32,
    h1: u32,
    h2: u32,
}

impl Fuse8 {
    #[inline]
    fn binary_fuse8_hash_batch(hash: u64, filter: &Fuse8) -> BinaryHashes {
        let mut ans = BinaryHashes::default();

        ans.h0 = binary_fuse_mulhi(hash, filter.segment_count_length.into()) as u32;
        ans.h1 = ans.h0 + filter.segment_length;
        ans.h2 = ans.h1 + filter.segment_length;
        ans.h1 ^= ((hash >> 18) as u32) & filter.segment_length_mask;
        ans.h2 ^= (hash as u32) & filter.segment_length_mask;
        ans
    }

    #[inline]
    fn binary_fuse8_hash(index: u32, hash: u64, filter: &Fuse8) -> u32 {
        let mut h = binary_fuse_mulhi(hash, filter.segment_count_length.into());
        h += (index * filter.segment_length) as u64;
        // keep the lower 36 bits
        let hh = hash & ((1_u64 << 36) - 1);
        // index 0: right shift by 36; index 1: right shift by 18; index 2: no shift
        h ^= (hh >> (36 - 18 * index)) & (filter.segment_length_mask as u64);

        h as u32
    }

    // allocate enough capacity for a set containing up to 'size' elements
    // caller is responsible to call binary_fuse8_free(filter)
    // size should be at least 2.
    pub fn new(size: u32) -> Option<Fuse8> {
        use std::cmp;

        match size {
            size if size <= 1 => None,
            size => {
                let arity = 3_u32;
                let segment_length =
                    cmp::min(binary_fuse_calculate_segment_length(arity, size), 262144);

                let segment_length_mask = segment_length - 1;
                let array_length = {
                    let size_factor = binary_fuse_calculate_size_factor(arity, size);
                    let cap = ((size as f64) * size_factor).round() as u32;
                    let n = (cap + segment_length - 1) / segment_length - (arity - 1);
                    (n + arity - 1) * segment_length
                };

                let segment_count = (array_length + segment_length - 1) / segment_length;
                let segment_count = if segment_count <= (arity - 1) {
                    1
                } else {
                    segment_count - (arity - 1)
                };

                let array_length = (segment_count + arity - 1) * segment_length;
                let segment_count_length = segment_count * segment_length;
                let finger_prints = {
                    let mut fp = Vec::with_capacity(array_length as usize);
                    fp.resize(array_length as usize, 0);
                    fp
                };

                let val = Fuse8 {
                    seed: u64::default(),
                    segment_length,
                    segment_length_mask,
                    segment_count,
                    segment_count_length,
                    finger_prints,
                };
                Some(val)
            }
        }
    }

    #[inline]
    pub fn size_of(&self) -> usize {
        std::mem::size_of::<Self>() + self.finger_prints.capacity()
    }

    // construct the filter, returns true on success, false on failure.
    // most likely, a failure is due to too high a memory usage
    // size is the number of keys
    // The caller is responsable for calling binary_fuse8_allocate(size,filter)
    // before. The caller is responsible to ensure that there are no duplicated
    // keys. The inner loop will run up to XOR_MAX_ITERATIONS times (default on
    // 100), it should never fail, except if there are duplicated keys. If it fails,
    // a return value of false is provided.
    pub fn populate(&mut self, keys: Vec<u64>) -> bool {
        let mut rng_counter = 0x726b2b9d438b9d4d_u64;
        let capacity = self.finger_prints.len();
        let size = keys.len();

        if size == 0 {
            return false;
        }

        self.seed = binary_fuse_rng_splitmix64(&mut rng_counter);
        let (mut reverse_order, mut reverse_h, mut alone, mut t2count, mut t2hash) =
            alloc_locals!(size, capacity);

        let mut block_bits: u32 = 1;
        while (1_u32 << block_bits) < self.segment_count {
            block_bits += 1;
        }

        let block = 1_u32 << block_bits;

        let mut start_pos: Vec<u32> = Vec::with_capacity(1 << block_bits);
        start_pos.resize(1 << block_bits, 0);

        let mut h012 = [0_u32; 5];

        reverse_order[size] = 1;
        let mut iter_n = 0;
        while iter_n <= XOR_MAX_ITERATIONS {
            for i in (0_u32..).take_while(|i| i < &block) {
                // important : i * size would overflow as a 32-bit number in some
                // cases.
                start_pos[i as usize] =
                    (((i as u64) * (size as u64)) >> block_bits) as u32;
            }

            let mask_block = (block - 1) as u64;
            for i in 0_usize..size {
                let hash: u64 = binary_fuse_murmur64(keys[i] + self.seed);
                let mut segment_index: u64 = hash >> (64 - block_bits);
                while reverse_order[start_pos[segment_index as usize] as usize] != 0 {
                    segment_index += 1;
                    segment_index &= mask_block;
                }
                reverse_order[start_pos[segment_index as usize] as usize] = hash;
                start_pos[segment_index as usize] += 1;
            }

            let mut error: isize = 0;
            for i in 0_usize..size {
                let hash: u64 = reverse_order[i];
                let h0: usize = Self::binary_fuse8_hash(0, hash, self) as usize;
                let h1: usize = Self::binary_fuse8_hash(1, hash, self) as usize;
                let h2: usize = Self::binary_fuse8_hash(2, hash, self) as usize;

                t2count[h0] += 4;
                t2hash[h0] ^= hash;

                t2count[h1] += 4;
                t2count[h1] ^= 1;
                t2hash[h1] ^= hash;

                t2count[h2] += 4;
                t2hash[h2] ^= hash;
                t2count[h2] ^= 2;

                error = if t2count[h0] < 4 { 1 } else { error };
                error = if t2count[h1] < 4 { 1 } else { error };
                error = if t2count[h2] < 4 { 1 } else { error };
            }

            if error > 0 {
                continue;
            }

            let mut q_size = 0_usize; // End of key addition

            // Add sets with one key to the queue.
            for i in 0_usize..capacity {
                alone[q_size] = i as u32;
                q_size += if (t2count[i] >> 2) == 1 { 1 } else { 0 };
            }

            let mut stack_size = 0_usize;

            while q_size > 0 {
                q_size -= 1;
                let index = alone[q_size] as usize;
                if (t2count[index] >> 2) == 1 {
                    let hash: u64 = t2hash[index];

                    //h012[0] = Self::binary_fuse8_hash(0, hash, self);
                    h012[1] = Self::binary_fuse8_hash(1, hash, self);
                    h012[2] = Self::binary_fuse8_hash(2, hash, self);
                    h012[3] = Self::binary_fuse8_hash(0, hash, self); // == h012[0];
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
            t2count.fill(0);
            t2hash.fill(0);

            self.seed = binary_fuse_rng_splitmix64(&mut rng_counter);

            iter_n += 1;
        }

        if iter_n > XOR_MAX_ITERATIONS {
            panic!("Too many iterations. Are all your keys unique?");
        }

        for i in (0_usize..size).rev() {
            // the hash of the key we insert next
            let hash: u64 = reverse_order[i];
            let xor2: u8 = binary_fuse8_fingerprint(hash) as u8;
            let found: usize = reverse_h[i] as usize;
            h012[0] = Self::binary_fuse8_hash(0, hash, self);
            h012[1] = Self::binary_fuse8_hash(1, hash, self);
            h012[2] = Self::binary_fuse8_hash(2, hash, self);
            h012[3] = h012[0];
            h012[4] = h012[1];
            self.finger_prints[h012[found] as usize] = xor2
                ^ self.finger_prints[h012[found + 1] as usize]
                ^ self.finger_prints[h012[found + 2] as usize];
        }

        true
    }

    // Report if the key is in the set, with false positive rate.
    #[inline]
    pub fn contain(&self, key: u64) -> bool {
        let hash = binary_fuse_mix_split(key, self.seed);
        let mut f = binary_fuse8_fingerprint(hash) as u8;
        let BinaryHashes { h0, h1, h2 } = Self::binary_fuse8_hash_batch(hash, self);
        f ^= self.finger_prints[h0 as usize]
            ^ self.finger_prints[h1 as usize]
            ^ self.finger_prints[h2 as usize];
        f == 0
    }
}
