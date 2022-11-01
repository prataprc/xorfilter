use criterion::{criterion_group, criterion_main, Criterion};

use rand::{prelude::random, rngs::StdRng, Rng, SeedableRng};
use xorfilter::Xor8;

use std::collections::hash_map::RandomState;

const SIZE: usize = 1_000_000;

fn generate_unique_keys(rng: &mut StdRng, size: usize) -> Vec<u64> {
    let mut keys: Vec<u64> = Vec::with_capacity(size);
    keys.resize(size, u64::default());

    for key in keys.iter_mut() {
        *key = rng.gen();
    }
    keys.sort_unstable();
    keys.dedup();

    for _i in 0..(size - keys.len()) {
        let key = rng.gen::<u64>();
        if !keys.contains(&key) {
            keys.push(key)
        }
    }

    keys
}

fn bench_xor8_populate_digests(c: &mut Criterion) {
    let seed: u64 = random();
    let mut rng = StdRng::seed_from_u64(seed);

    let keys = generate_unique_keys(&mut rng, SIZE);

    c.bench_function("xor8_populate_digests", |b| {
        b.iter(|| {
            let mut filter = Xor8::<RandomState>::new();
            filter.populate_keys(&keys);
            filter.build().expect("failed build");
        })
    });
}

fn bench_xor8_build_from_digests(c: &mut Criterion) {
    let seed: u64 = random();
    let mut rng = StdRng::seed_from_u64(seed);

    let keys = generate_unique_keys(&mut rng, SIZE);

    c.bench_function("xor8_build_from_digests", |b| {
        b.iter(|| {
            let mut filter = Xor8::<RandomState>::new();
            filter.build_from_digests(&keys).expect("failed build");
        })
    });
}

fn bench_xor8_populate(c: &mut Criterion) {
    let seed: u64 = random();
    let mut rng = StdRng::seed_from_u64(seed);

    let keys = generate_unique_keys(&mut rng, SIZE);

    c.bench_function("xor8_populate", |b| {
        b.iter(|| {
            let mut filter = Xor8::<RandomState>::new();
            filter.populate(&keys);
            filter.build().expect("failed build");
        })
    });
}

fn bench_xor8_insert(c: &mut Criterion) {
    let seed: u64 = random();
    let mut rng = StdRng::seed_from_u64(seed);

    let keys = generate_unique_keys(&mut rng, SIZE);

    c.bench_function("xor8_insert", |b| {
        b.iter(|| {
            let mut filter = Xor8::<RandomState>::new();
            keys.iter().for_each(|key| filter.insert(key));
            filter.build().expect("failed build");
        })
    });
}

fn bench_xor8_contains(c: &mut Criterion) {
    let seed: u64 = random();
    let mut rng = StdRng::seed_from_u64(seed);

    let keys = generate_unique_keys(&mut rng, SIZE);

    let filter = {
        let mut filter = Xor8::<RandomState>::new();
        filter.populate(&keys);
        filter.build().expect("failed build");
        filter
    };

    let mut n = 0;
    c.bench_function("xor8_contains", |b| {
        b.iter(|| {
            filter.contains(&keys[n % keys.len()]);
            n += 1;
        })
    });
}

fn bench_xor8_contains_digest(c: &mut Criterion) {
    let seed: u64 = random();
    let mut rng = StdRng::seed_from_u64(seed);

    let keys = generate_unique_keys(&mut rng, SIZE);

    let filter = {
        let mut filter = Xor8::<RandomState>::new();
        filter.populate(&keys);
        filter.build().expect("failed build");
        filter
    };

    let mut n = 0;
    c.bench_function("xor8_contains_digest", |b| {
        b.iter(|| {
            filter.contains_digest(keys[n % keys.len()]);
            n += 1;
        })
    });
}

criterion_group!(
    benches,
    bench_xor8_populate_digests,
    bench_xor8_build_from_digests,
    bench_xor8_populate,
    bench_xor8_insert,
    bench_xor8_contains,
    bench_xor8_contains_digest,
);

criterion_main!(benches);
