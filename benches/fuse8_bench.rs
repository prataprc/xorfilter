use criterion::{criterion_group, criterion_main, Criterion};

use rand::{prelude::random, rngs::SmallRng, Rng, SeedableRng};
use xorfilter::Fuse8;

fn bench_fuse8_populate_keys_100000(c: &mut Criterion) {
    let seed: u128 = random();
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let testsize = 10_000_000;
    let mut keys: Vec<u64> = Vec::with_capacity(testsize);
    keys.resize(testsize, Default::default());
    for key in keys.iter_mut() {
        *key = rng.gen();
    }

    c.bench_function("populate_keys_100000", |b| {
        b.iter(|| {
            let mut filter = Fuse8::new(testsize as u32).unwrap();
            filter.populate(&keys);
        })
    });
}

fn bench_fuse8_contains_100000(c: &mut Criterion) {
    let seed: u128 = random();
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let testsize = 100_000;
    let mut keys: Vec<u64> = Vec::with_capacity(testsize);
    keys.resize(testsize, Default::default());
    for key in keys.iter_mut() {
        *key = rng.gen();
    }

    let mut filter = Fuse8::new(testsize as u32).unwrap();
    filter.populate(&keys);

    let mut n = 0;
    c.bench_function("bench_contains_100000", |b| {
        b.iter(|| {
            filter.contains(keys[n % keys.len()]);
            n += 1;
        })
    });
}

criterion_group!(
    benches,
    bench_fuse8_populate_keys_100000,
    bench_fuse8_contains_100000,
);

criterion_main!(benches);
