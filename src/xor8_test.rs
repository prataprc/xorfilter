use super::*;
use rand::{prelude::random, rngs::SmallRng, Rng, SeedableRng};

fn generate_keys(rng: &mut SmallRng, size: usize) -> Vec<u64> {
    let mut keys: Vec<u64> = Vec::with_capacity(size);
    keys.resize(size, Default::default());

    for key in keys.iter_mut() {
        *key = rng.gen();
    }
    keys
}

fn test_xor8_build<H>(name: &str)
where
    H: Default + BuildHasher,
{
    let seed: u128 = random();
    println!("test_xor8_build<{}> seed {}", name, seed);
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let mut filter = Xor8::<RandomState>::new();

    // populate api
    let mut keys = generate_keys(&mut rng, 2_000_000);
    filter.populate(&keys);
    // populate_keys api
    keys.extend({
        let keys = generate_keys(&mut rng, 2_000_000);
        let digests: Vec<u64> = keys
            .iter()
            .map(|k| {
                let mut hasher = filter.get_hasher();
                k.hash(&mut hasher);
                hasher.finish()
            })
            .collect();
        filter.populate_keys(&digests);
        keys
    });
    // insert api
    keys.extend({
        let keys = generate_keys(&mut rng, 2_000_000);
        keys.iter().for_each(|key| filter.insert(key));
        keys
    });
    let test_size = 6_000_000_usize;

    filter.build();

    // contains api
    for key in keys.iter() {
        assert!(filter.contains(key), "key {} not present", key);
    }
    // contains_key api
    for key in keys.iter() {
        let digest = {
            let mut hasher = filter.get_hasher();
            key.hash(&mut hasher);
            hasher.finish()
        };
        assert!(filter.contains_key(digest), "key {} not present", key);
    }

    // print some statistics
    let (falsesize, mut matches) = (10_000_000, 0_f64);
    let bpv = (filter.finger_prints.len() as f64) * 8.0 / (test_size as f64);
    println!("test_xor8_build<{}> bits per entry {} bits", name, bpv);
    assert!(bpv < 10.0, "bpv({}) >= 10.0", bpv);

    for _ in 0..falsesize {
        if filter.contains(&rng.gen::<u64>()) {
            matches += 1_f64;
        }
    }

    let fpp = matches * 100.0 / (falsesize as f64);
    println!("test_xor8_build<{}> false positive rate {}%", name, fpp);
    assert!(fpp < 0.40, "fpp({}) >= 0.40", fpp);
}

fn test_xor8_build_keys<H>(name: &str)
where
    H: Default + BuildHasher,
{
    let seed: u128 = random();
    println!("test_xor8_build_keys<{}> seed {}", name, seed);
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let mut filter = Xor8::<H>::new();

    // build_keys api
    let keys = generate_keys(&mut rng, 2_000_000);
    let digests: Vec<u64> = keys
        .iter()
        .map(|k| {
            let mut hasher = filter.get_hasher();
            k.hash(&mut hasher);
            hasher.finish()
        })
        .collect();
    filter.build_keys(&digests);
    let test_size = 2_000_000_usize;

    // contains api
    for key in keys.iter() {
        assert!(filter.contains(key), "key {} not present", key);
    }
    // contains_key api
    for digest in digests.into_iter() {
        assert!(filter.contains_key(digest), "digest {} not present", digest);
    }

    // print some statistics
    let (falsesize, mut matches) = (10_000_000, 0_f64);
    let bpv = (filter.finger_prints.len() as f64) * 8.0 / (test_size as f64);
    println!("test_xor8_build_keys<{}> bits per entry {} bits", name, bpv);
    assert!(bpv < 10.0, "bpv({}) >= 10.0", bpv);

    for _ in 0..falsesize {
        if filter.contains(&rng.gen::<u64>()) {
            matches += 1_f64;
        }
    }

    let fpp = matches * 100.0 / (falsesize as f64);
    println!(
        "test_xor8_build_keys<{}> false positive rate {}%",
        name, fpp
    );
    assert!(fpp < 0.40, "fpp({}) >= 0.40", fpp);
}

#[test]
fn test_xor8() {
    test_xor8_build::<RandomState>("RandomState");
    test_xor8_build::<BuildHasherDefault>("BuildHasherDefault");
    test_xor8_build_keys::<RandomState>("RandomState");
    test_xor8_build_keys::<BuildHasherDefault>("BuildHasherDefault");
}
