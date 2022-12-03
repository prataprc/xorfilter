use std::collections::hash_map::RandomState;

use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use rand::prelude::random;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use xorfilter::Fuse16;

const SIZE: usize = 1_000_000;

fn generate_unique_keys(rng: &mut StdRng, size: usize) -> Vec<u64> {
    let mut keys: Vec<u64> = Vec::with_capacity(size);
    keys.resize(size, Default::default());

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

fn bench_fuse16_populate_keys(c: &mut Criterion) {
    let seed: u64 = random();
    println!("bench_fuse16_populate_keys seed:{}", seed);
    let mut rng = StdRng::seed_from_u64(seed);

    let keys = generate_unique_keys(&mut rng, SIZE);

    c.bench_function("fuse16_populate_keys", |b| {
        b.iter(|| {
            let mut filter = Fuse16::<RandomState>::new(keys.len() as u32);
            filter.populate_keys(&keys);
            filter.build().expect("failed build");
        })
    });
}

fn bench_fuse16_build_keys(c: &mut Criterion) {
    let seed: u64 = random();
    println!("bench_fuse16_build_keys seed:{}", seed);
    let mut rng = StdRng::seed_from_u64(seed);

    let keys = generate_unique_keys(&mut rng, SIZE);

    c.bench_function("fuse16_build_keys", |b| {
        b.iter(|| {
            let mut filter = Fuse16::<RandomState>::new(keys.len() as u32);
            filter.build_keys(&keys).expect("failed build");
        })
    });
}

fn bench_fuse16_populate(c: &mut Criterion) {
    let seed: u64 = random();
    println!("bench_fuse16_populate seed:{}", seed);
    let mut rng = StdRng::seed_from_u64(seed);

    let keys = generate_unique_keys(&mut rng, SIZE);

    c.bench_function("fuse16_populate", |b| {
        b.iter(|| {
            let mut filter = Fuse16::<RandomState>::new(keys.len() as u32);
            filter.populate(&keys);
            filter.build().expect("failed build");
        })
    });
}

fn bench_fuse16_insert(c: &mut Criterion) {
    let seed: u64 = random();
    println!("bench_fuse16_insert seed:{}", seed);
    let mut rng = StdRng::seed_from_u64(seed);

    let keys = generate_unique_keys(&mut rng, SIZE);

    c.bench_function("fuse16_insert", |b| {
        b.iter(|| {
            let mut filter = Fuse16::<RandomState>::new(keys.len() as u32);
            keys.iter().for_each(|key| filter.insert(key));
            filter.build().expect("failed build");
        })
    });
}

fn bench_fuse16_contains(c: &mut Criterion) {
    let seed: u64 = random();
    println!("bench_fuse16_contains seed:{}", seed);
    let mut rng = StdRng::seed_from_u64(seed);

    let keys = generate_unique_keys(&mut rng, SIZE);

    let filter = {
        let mut filter = Fuse16::<RandomState>::new(keys.len() as u32);
        filter.populate(&keys);
        filter.build().expect("failed build");
        filter
    };

    let mut n = 0;
    c.bench_function("fuse16_contains", |b| {
        b.iter(|| {
            filter.contains(&keys[n % keys.len()]);
            n += 1;
        })
    });
}

fn bench_fuse16_contains_key(c: &mut Criterion) {
    let seed: u64 = random();
    println!("bench_fuse16_contains_key seed:{}", seed);
    let mut rng = StdRng::seed_from_u64(seed);

    let keys = generate_unique_keys(&mut rng, SIZE);

    let filter = {
        let mut filter = Fuse16::<RandomState>::new(keys.len() as u32);
        filter.populate(&keys);
        filter.build().expect("failed build");
        filter
    };

    let mut n = 0;
    c.bench_function("fuse16_contains_key", |b| {
        b.iter(|| {
            filter.contains_key(keys[n % keys.len()]);
            n += 1;
        })
    });
}

criterion_group!(
    benches,
    bench_fuse16_populate_keys,
    bench_fuse16_build_keys,
    bench_fuse16_populate,
    bench_fuse16_insert,
    bench_fuse16_contains,
    bench_fuse16_contains_key,
);

criterion_main!(benches);
