use criterion::{criterion_group, criterion_main, Criterion};

use rand::{prelude::random, rngs::SmallRng, Rng, SeedableRng};
use xorfilter::Fuse8;

use std::collections::hash_map::RandomState;

const SIZE: usize = 10_000_000;

fn generate_keys(rng: &mut SmallRng, size: usize) -> Vec<u64> {
    let mut keys: Vec<u64> = Vec::with_capacity(size);
    keys.resize(size, Default::default());
    for key in keys.iter_mut() {
        *key = rng.gen();
    }
    keys
}

fn bench_fuse8_populate_keys(c: &mut Criterion) {
    let seed: u128 = random();
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let keys = generate_keys(&mut rng, SIZE);

    c.bench_function("fuse8_populate_keys", |b| {
        b.iter(|| {
            let mut filter = Fuse8::<RandomState>::new();
            filter.populate_keys(&keys);
            filter.build();
        })
    });
}

fn bench_fuse8_build_keys(c: &mut Criterion) {
    let seed: u128 = random();
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let keys = generate_keys(&mut rng, SIZE);

    c.bench_function("fuse8_build_keys", |b| {
        b.iter(|| {
            let mut filter = Fuse8::<RandomState>::new();
            filter.build_keys(&keys);
        })
    });
}

fn bench_fuse8_populate(c: &mut Criterion) {
    let seed: u128 = random();
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let keys = generate_keys(&mut rng, SIZE);

    c.bench_function("fuse8_populate", |b| {
        b.iter(|| {
            let mut filter = Fuse8::<RandomState>::new();
            filter.populate(&keys);
            filter.build();
        })
    });
}

fn bench_fuse8_insert(c: &mut Criterion) {
    let seed: u128 = random();
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let keys = generate_keys(&mut rng, SIZE);

    c.bench_function("fuse8_insert", |b| {
        b.iter(|| {
            let mut filter = Fuse8::<RandomState>::new();
            keys.iter().for_each(|key| filter.insert(key));
            filter.build();
        })
    });
}

fn bench_fuse8_contains(c: &mut Criterion) {
    let seed: u128 = random();
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let keys = generate_keys(&mut rng, SIZE);

    let filter = {
        let mut filter = Fuse8::<RandomState>::new();
        filter.populate(&keys);
        filter.build();
        filter
    };

    let mut n = 0;
    c.bench_function("fuse8_contains", |b| {
        b.iter(|| {
            filter.contains(&keys[n % keys.len()]);
            n += 1;
        })
    });
}

fn bench_fuse8_contains_key(c: &mut Criterion) {
    let seed: u128 = random();
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let keys = generate_keys(&mut rng, SIZE);

    let filter = {
        let mut filter = Fuse8::<RandomState>::new();
        filter.populate(&keys);
        filter.build();
        filter
    };

    let mut n = 0;
    c.bench_function("fuse8_contains_key", |b| {
        b.iter(|| {
            filter.contains_key(keys[n % keys.len()]);
            n += 1;
        })
    });
}

criterion_group!(
    benches,
    bench_fuse8_populate_keys,
    bench_fuse8_build_keys,
    bench_fuse8_populate,
    bench_fuse8_insert,
    bench_fuse8_contains,
    bench_fuse8_contains_key,
);

criterion_main!(benches);
