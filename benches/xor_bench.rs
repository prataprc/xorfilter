use criterion::{criterion_group, criterion_main, Criterion};

use rand::{prelude::random, rngs::SmallRng, Rng, SeedableRng};
use xorfilter::Xor8;

use std::collections::hash_map::RandomState;

fn bench_xor8_populate_keys_100000(c: &mut Criterion) {
    let seed: u128 = random();
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let testsize = 100_000;
    let mut keys: Vec<u64> = Vec::with_capacity(testsize);
    keys.resize(testsize, Default::default());
    for key in keys.iter_mut() {
        *key = rng.gen();
    }

    c.bench_function("populate_keys_100000", |b| {
        b.iter(|| {
            let mut filter = Xor8::<RandomState>::new();
            filter.populate_keys(&keys);
            filter.build();
        })
    });
}

fn bench_xor8_build_keys_100000(c: &mut Criterion) {
    let seed: u128 = random();
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let testsize = 100_000;
    let mut keys: Vec<u64> = Vec::with_capacity(testsize);
    keys.resize(testsize, Default::default());
    for key in keys.iter_mut() {
        *key = rng.gen();
    }

    c.bench_function("bench_build_keys_100000", |b| {
        b.iter(|| {
            let mut filter = Xor8::<RandomState>::new();
            filter.build_keys(&keys);
        })
    });
}

fn bench_xor8_populate_100000(c: &mut Criterion) {
    let seed: u128 = random();
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let testsize = 100_000;
    let mut keys: Vec<u64> = Vec::with_capacity(testsize);
    keys.resize(testsize, Default::default());
    for key in keys.iter_mut() {
        *key = rng.gen();
    }

    c.bench_function("bench_populate_100000", |b| {
        b.iter(|| {
            let mut filter = Xor8::<RandomState>::new();
            filter.populate(&keys);
            filter.build();
        })
    });
}

fn bench_xor8_insert_100000(c: &mut Criterion) {
    let seed: u128 = random();
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let testsize = 100_000;
    let mut keys: Vec<u64> = Vec::with_capacity(testsize);
    keys.resize(testsize, Default::default());
    for key in keys.iter_mut() {
        *key = rng.gen();
    }

    c.bench_function("bench_insert_100000", |b| {
        b.iter(|| {
            let mut filter = Xor8::<RandomState>::new();
            keys.iter().for_each(|key| filter.insert(key));
            filter.build();
        })
    });
}

fn bench_xor8_contains_100000(c: &mut Criterion) {
    let seed: u128 = random();
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let testsize = 100_000;
    let mut keys: Vec<u64> = Vec::with_capacity(testsize);
    keys.resize(testsize, Default::default());
    for key in keys.iter_mut() {
        *key = rng.gen();
    }

    let filter = {
        let mut filter = Xor8::<RandomState>::new();
        filter.populate(&keys);
        filter.build();
        filter
    };

    let mut n = 0;
    c.bench_function("bench_contains_100000", |b| {
        b.iter(|| {
            filter.contains(&keys[n % keys.len()]);
            n += 1;
        })
    });
}

fn bench_xor8_contains_key_100000(c: &mut Criterion) {
    let seed: u128 = random();
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let testsize = 100_000;
    let mut keys: Vec<u64> = Vec::with_capacity(testsize);
    keys.resize(testsize, Default::default());
    for key in keys.iter_mut() {
        *key = rng.gen();
    }

    let filter = {
        let mut filter = Xor8::<RandomState>::new();
        filter.populate(&keys);
        filter.build();
        filter
    };

    let mut n = 0;
    c.bench_function("bench_contains_key_100000", |b| {
        b.iter(|| {
            filter.contains_key(keys[n % keys.len()]);
            n += 1;
        })
    });
}

criterion_group!(
    benches,
    bench_xor8_populate_keys_100000,
    bench_xor8_build_keys_100000,
    bench_xor8_populate_100000,
    bench_xor8_insert_100000,
    bench_xor8_contains_100000,
    bench_xor8_contains_key_100000
);

criterion_main!(benches);
