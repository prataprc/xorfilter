use std::collections::hash_map::RandomState;

use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use rand::prelude::random;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use xorfilter::xor8::Xor8Builder;

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

    let digests = generate_unique_keys(&mut rng, SIZE);

    c.bench_function("xor8_populate_digests", |b| {
        b.iter(|| {
            let mut builder = Xor8Builder::<RandomState>::new();
            builder.populate_digests(&digests);
            let _filter = criterion::black_box(builder.build().expect("failed build"));
        })
    });
}

fn bench_xor8_build_from_digests(c: &mut Criterion) {
    let seed: u64 = random();
    let mut rng = StdRng::seed_from_u64(seed);

    let keys = generate_unique_keys(&mut rng, SIZE);

    c.bench_function("xor8_build_from_digests", |b| {
        b.iter(|| {
            let mut builder = Xor8Builder::<RandomState>::new();
            let _filter = criterion::black_box(
                builder.build_from_digests(&keys).expect("failed build"),
            );
        })
    });
}

fn bench_xor8_populate(c: &mut Criterion) {
    let seed: u64 = random();
    let mut rng = StdRng::seed_from_u64(seed);

    let keys = generate_unique_keys(&mut rng, SIZE);

    c.bench_function("xor8_populate", |b| {
        b.iter(|| {
            let mut builder = Xor8Builder::<RandomState>::new();
            builder.populate(&keys);
            let _filter = criterion::black_box(builder.build().expect("failed build"));
        })
    });
}

fn bench_xor8_insert(c: &mut Criterion) {
    let seed: u64 = random();
    let mut rng = StdRng::seed_from_u64(seed);

    let keys = generate_unique_keys(&mut rng, SIZE);

    c.bench_function("xor8_insert", |b| {
        b.iter(|| {
            let mut builder = Xor8Builder::<RandomState>::new();
            keys.iter().for_each(|key| builder.insert(key));
            let _f = criterion::black_box(builder.build().expect("failed build"));
        })
    });
}

fn bench_xor8_contains(c: &mut Criterion) {
    let seed: u64 = random();
    let mut rng = StdRng::seed_from_u64(seed);

    let keys = generate_unique_keys(&mut rng, SIZE);

    let filter = {
        let mut builder = Xor8Builder::<RandomState>::new();
        builder.populate(&keys);
        builder.build().expect("failed build")
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
        let mut builder = Xor8Builder::<RandomState>::new();
        builder.populate(&keys);
        builder.build().expect("failed build")
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
