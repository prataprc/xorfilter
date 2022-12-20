use rand::distributions::Distribution;
use rand::distributions::Standard;
use rand::prelude::random;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

use super::*;

fn generate_unique_keys<K>(prefix: &str, rng: &mut StdRng, size: usize) -> Vec<K>
where
    K: Clone + Default + Ord,
    Standard: Distribution<K>,
{
    let mut keys: Vec<K> = Vec::with_capacity(size);
    keys.resize(size, K::default());

    for key in keys.iter_mut() {
        *key = rng.gen();
    }
    keys.sort_unstable();

    let mut ks = keys.clone();
    ks.dedup();
    println!("{} number of duplicates {}", prefix, size - ks.len());

    keys
}

fn test_fuse8_build<H, K>(name: &str, seed: u64, size: u32)
where
    H: Default + BuildHasher,
    K: Clone + Default + Ord + Hash + std::fmt::Display,
    Standard: Distribution<K>,
{
    use std::cmp;

    let mut rng = StdRng::seed_from_u64(seed);

    let keys = generate_unique_keys(name, &mut rng, size as usize);

    let size = keys.len() as u32;
    let (x, y) = {
        let size = size as usize;
        (size / 3, size / 3)
    };
    let (keys1, keys2, keys3) = (&keys[0..x], &keys[x..x + y], &keys[x + y..]);

    println!("test_fuse8_build<{}> size:{}", name, size);

    let mut filter = Fuse8::<H>::new(size);

    // populate api
    filter.populate(keys1);
    // populate_keys api
    let digests: Vec<u64> = keys2
        .iter()
        .map(|k| {
            let mut hasher = filter.get_hasher();
            k.hash(&mut hasher);
            hasher.finish()
        })
        .collect();
    filter.populate_keys(&digests);
    // insert api
    keys3.iter().for_each(|key| filter.insert(key));

    filter.build().expect("failed to build fuse16 filter");

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
    let (falsesize, mut matches) = (cmp::min(size * 10, 10_000_000), 0_f64);
    let bpv = (filter.finger_prints.len() as f64) * 8.0 / (keys.len() as f64);
    println!("test_fuse8_build<{}> bits per entry {} bits", name, bpv);

    for _ in 0..falsesize {
        let k = rng.gen::<K>();
        let ok = filter.contains(&k);
        match keys.binary_search(&k) {
            Ok(_) if !ok => panic!("false negative {}", k),
            Ok(_) => (),
            Err(_) if ok => matches += 1_f64,
            Err(_) => (),
        }
    }

    let fpp = matches * 100.0 / (falsesize as f64);
    println!("test_fuse8_build<{}> false positive rate {}%", name, fpp);

    if size > 100_000 {
        assert!(bpv < 12.0, "bpv({}) >= 12.0", bpv);
        assert!(fpp < 0.4, "fpp({}) >= 0.4", fpp);
    }
}

fn test_fuse8_build_keys<H, K>(name: &str, seed: u64, size: u32)
where
    H: Default + BuildHasher,
    K: Clone + Default + Ord + Hash + std::fmt::Display,
    Standard: Distribution<K>,
{
    use std::cmp;

    let mut rng = StdRng::seed_from_u64(seed);

    let keys = generate_unique_keys(name, &mut rng, size as usize);
    let size = keys.len() as u32;

    println!("test_fuse8_build_keys<{}> size:{}", name, size);

    let mut filter = Fuse8::<H>::new(size);

    // build_keys api
    let digests: Vec<u64> = keys
        .iter()
        .map(|k| {
            let mut hasher = filter.get_hasher();
            k.hash(&mut hasher);
            hasher.finish()
        })
        .collect();

    filter.build_keys(&digests).expect("failed to build fuse16 filter");

    // contains api
    for key in keys.iter() {
        assert!(filter.contains(key), "key {} not present", key);
    }
    // contains_key api
    for digest in digests.into_iter() {
        assert!(filter.contains_key(digest), "digest {} not present", digest);
    }

    // print some statistics
    let (falsesize, mut matches) = (cmp::min(size * 10, 10_000_000), 0_f64);
    let bpv = (filter.finger_prints.len() as f64) * 8.0 / (keys.len() as f64);
    println!(
        "test_fuse8_build_keys<{}> bits per entry {} bits",
        name, bpv
    );

    for _ in 0..falsesize {
        let k = rng.gen::<K>();
        let ok = filter.contains(&k);
        match keys.binary_search(&k) {
            Ok(_) if !ok => panic!("false negative {}", k),
            Ok(_) => (),
            Err(_) if ok => matches += 1_f64,
            Err(_) => (),
        }
    }

    let fpp = matches * 100.0 / (falsesize as f64);
    println!(
        "test_fuse8_build_keys<{}> false positive rate {}%",
        name, fpp
    );

    if size > 100_000 {
        assert!(bpv < 12.0, "bpv({}) >= 12.0", bpv);
        assert!(fpp < 0.4, "fpp({}) >= 0.4", fpp);
    }
}

#[test]
fn test_fuse8_u8() {
    let mut seed: u64 = [6509898893809465102_u64, random()][random::<usize>() % 2];
    println!("test_fuse8_u8 seed:{}", seed);

    for size in [0, 1, 2, 10, 100].iter() {
        seed = seed.wrapping_add(*size as u64);
        test_fuse8_build::<RandomState, u8>("RandomState,u8", seed, *size);
        test_fuse8_build::<BuildHasherDefault, u8>("BuildHasherDefault,u8", seed, *size);
        test_fuse8_build_keys::<RandomState, u8>("RandomState,u8", seed, *size);
        test_fuse8_build_keys::<BuildHasherDefault, u8>(
            "BuildHasherDefault,u8",
            seed,
            *size,
        );
    }
}

#[test]
fn test_fuse8_u16() {
    let mut seed: u64 = random();
    println!("test_fuse8_u16 seed:{}", seed);

    for size in [0, 1, 2, 10, 100, 500].iter() {
        seed = seed.wrapping_add(*size as u64);
        test_fuse8_build::<RandomState, u16>("RandomState,16", seed, *size);
        test_fuse8_build::<BuildHasherDefault, u16>("BuildHasherDefault,16", seed, *size);
        test_fuse8_build_keys::<RandomState, u16>("RandomState,16", seed, *size);
        test_fuse8_build_keys::<BuildHasherDefault, u16>(
            "BuildHasherDefault,16",
            seed,
            *size,
        );
    }
}

#[test]
fn test_fuse8_u64() {
    let mut seed: u64 = random();
    println!("test_fuse8_u64 seed:{}", seed);

    for size in [0, 1, 2, 10, 1000, 10_000, 100_000, 1_000_000, 10_000_000].iter() {
        seed = seed.wrapping_add(*size as u64);
        test_fuse8_build::<RandomState, u64>("RandomState,64", seed, *size);
        test_fuse8_build::<BuildHasherDefault, u64>("BuildHasherDefault,64", seed, *size);
        test_fuse8_build_keys::<RandomState, u64>("RandomState,64", seed, *size);
        test_fuse8_build_keys::<BuildHasherDefault, u64>(
            "BuildHasherDefault,64",
            seed,
            *size,
        );
    }
}

#[test]
fn test_fuse8_duplicates() {
    println!("test_fuse8_duplicates");

    let keys = vec![102, 123, 1242352, 12314, 124235, 1231234, 12414, 1242352];

    let mut filter = Fuse8::<RandomState>::new(keys.len() as u32);

    filter.build_keys(&keys).expect("build with duplicate keys failed");

    // contains api
    for key in keys.iter() {
        assert!(filter.contains_key(*key), "key {} not present", key);
    }
}

#[test]
#[ignore]
fn test_fuse8_billion() {
    let seed: u64 = random();
    println!("test_fuse8_billion seed:{}", seed);

    let size = 1_000_000_000;
    test_fuse8_build::<RandomState, u64>("RandomState,u64", seed, size);
    test_fuse8_build::<BuildHasherDefault, u64>("BuildHasherDefault,u64", seed, size);
    test_fuse8_build_keys::<RandomState, u64>("RandomState,u64", seed, size);
    test_fuse8_build_keys::<BuildHasherDefault, u64>(
        "BuildHasherDefault,u64",
        seed,
        size,
    );
}

#[cfg(feature = "cbordata")]
#[test]
fn test_fuse8_cbor() {
    let seed: u64 = random();
    println!("test_fuse8_cbor seed:{}", seed);
    let mut rng = StdRng::seed_from_u64(seed);

    let keys: Vec<u64> = (0..100_000).map(|_| rng.gen::<u64>()).collect();

    let filter = {
        let mut filter = Fuse8::<BuildHasherDefault>::new(keys.len() as u32);
        filter.populate(&keys);
        filter.build().expect("fail building fuse8 filter");
        filter
    };

    for key in keys.iter() {
        assert!(filter.contains(key), "key {} not present", key);
    }

    let filter = {
        let val = filter.into_cbor().unwrap();
        Fuse8::<BuildHasherDefault>::from_cbor(val).unwrap()
    };

    for key in keys.iter() {
        assert!(filter.contains(key), "key {} not present", key);
    }
}
