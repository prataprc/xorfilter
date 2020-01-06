#![feature(test)]
extern crate test;

use rand::{prelude::random, rngs::SmallRng, Rng, SeedableRng};

use test::Bencher;

use xorfilter::Xor8;

#[bench]
fn bench_populate_100000(b: &mut Bencher) {
    let seed: u128 = random();
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let testsize = 10000;
    let mut keys: Vec<u64> = Vec::with_capacity(testsize);
    keys.resize(testsize, Default::default());
    for i in 0..keys.len() {
        keys[i] = rng.gen();
    }

    b.iter(|| Xor8::new(&keys));
}

#[bench]
fn bench_contains_100000(b: &mut Bencher) {
    let seed: u128 = random();
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let testsize = 10000;
    let mut keys: Vec<u64> = Vec::with_capacity(testsize);
    keys.resize(testsize, Default::default());
    for i in 0..keys.len() {
        keys[i] = rng.gen();
    }

    let filter = Xor8::new(&keys);
    let mut n = 0;
    b.iter(|| {
        filter.contains(keys[n % keys.len()]);
        n += 1;
    });
}
