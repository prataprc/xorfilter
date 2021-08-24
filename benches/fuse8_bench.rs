use criterion::{criterion_group, criterion_main, Criterion};

use rand::{prelude::random, rngs::SmallRng, Rng, SeedableRng};
use xorfilter::Fuse8;

const SIZE: usize = 10_000_000;

fn bench_fuse8_populate_keys(c: &mut Criterion) {
    let seed: u128 = random();
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let mut keys: Vec<u64> = Vec::with_capacity(SIZE);
    keys.resize(SIZE, Default::default());
    for key in keys.iter_mut() {
        *key = rng.gen();
    }

    c.bench_function("fuse8_populate_keys", |b| {
        b.iter(|| {
            let mut filter = Fuse8::new(SIZE as u32).unwrap();
            filter.populate(&keys);
        })
    });
}

fn bench_fuse8_contains(c: &mut Criterion) {
    let seed: u128 = random();
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let mut keys: Vec<u64> = Vec::with_capacity(SIZE);
    keys.resize(SIZE, Default::default());
    for key in keys.iter_mut() {
        *key = rng.gen();
    }

    let mut filter = Fuse8::new(SIZE as u32).unwrap();
    filter.populate(&keys);

    let mut n = 0;
    c.bench_function("fuse8_contains", |b| {
        b.iter(|| {
            filter.contains(keys[n % keys.len()]);
            n += 1;
        })
    });
}

criterion_group!(benches, bench_fuse8_populate_keys, bench_fuse8_contains,);

criterion_main!(benches);
