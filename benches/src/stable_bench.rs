// benchmark for stable bloom filter
// see https://github.com/u2/stable-bloom-filter/tree/master/benches

use criterion::{criterion_group, criterion_main, Criterion, Fun};
use stable_bloom_filter::stable::StableBloomFilter;
use stable_bloom_filter::Filter;

fn bench(c: &mut Criterion) {
    let add = Fun::new("Add", |b, _| {
        let mut s = StableBloomFilter::new_default(200, 0.01);
        let mut data = Vec::new();
        for i in 0..100_000 {
            data.push(i.to_string().into_bytes());
        }

        b.iter(|| {
            for i in data.iter() {
                s.add(i);
            }
        })
    });

    let test = Fun::new("Test", |b, _| {
        let s = StableBloomFilter::new_default(200, 0.01);
        let mut data = Vec::new();
        for i in 0..100_000 {
            data.push(i.to_string().into_bytes());
        }

        b.iter(|| {
            for i in data.iter() {
                s.test(i);
            }
        })
    });

    let test_and_add = Fun::new("TestAndAdd", |b, _| {
        let mut s = StableBloomFilter::new_default(200, 0.01);
        let mut data = Vec::new();
        for i in 0..100_000 {
            data.push(i.to_string().into_bytes());
        }

        b.iter(|| {
            for i in data.iter() {
                s.test_and_add(i);
            }
        })
    });

    let functions = vec![add, test, test_and_add];
    c.bench_functions("StableBloomFilter", functions, 0);
}

criterion_group!(benches, bench);
criterion_main!(benches);
