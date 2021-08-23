use super::*;
use rand::{prelude::random, rngs::SmallRng, Rng, SeedableRng};

#[test]
fn test_xor8_basic1() {
    let seed: u128 = random();
    println!("test_basic1 seed {}", seed);
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let testsize = 1_000_000;
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

    for key in keys.iter() {
        assert!(filter.contains(key), "key {} not present", key);
    }

    let (falsesize, mut matches) = (10_000_000, 0_f64);
    let bpv = (filter.finger_prints.len() as f64) * 8.0 / (testsize as f64);
    println!("test_basic1 bits per entry {} bits", bpv);
    assert!(bpv < 10.0, "bpv({}) >= 10.0", bpv);

    for _ in 0..falsesize {
        if filter.contains(&rng.gen::<u64>()) {
            matches += 1_f64;
        }
    }
    let fpp = matches * 100.0 / (falsesize as f64);
    println!("test_basic1 false positive rate {}%", fpp);
    assert!(fpp < 0.40, "fpp({}) >= 0.40", fpp);
}

#[test]
fn test_xor8_basic2() {
    let seed: u128 = random();
    println!("test_basic2 seed {}", seed);
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let testsize = 1_000_000;
    let mut keys: Vec<u64> = Vec::with_capacity(testsize);
    keys.resize(testsize, Default::default());
    for key in keys.iter_mut() {
        *key = rng.gen();
    }

    let filter = {
        let mut filter = Xor8::<RandomState>::new();
        filter.populate_keys(&keys);
        filter.build();
        filter
    };

    for key in keys.into_iter() {
        assert!(filter.contains_key(key), "key {} not present", key);
    }

    let (falsesize, mut matches) = (10_000_000, 0_f64);
    let bpv = (filter.finger_prints.len() as f64) * 8.0 / (testsize as f64);
    println!("test_basic2 bits per entry {} bits", bpv);
    assert!(bpv < 10.0, "bpv({}) >= 10.0", bpv);

    for _ in 0..falsesize {
        if filter.contains(&rng.gen::<u64>()) {
            matches += 1_f64;
        }
    }
    let fpp = matches * 100.0 / (falsesize as f64);
    println!("test_basic2 false positive rate {}%", fpp);
    assert!(fpp < 0.40, "fpp({}) >= 0.40", fpp);
}

#[test]
fn test_xor8_basic3() {
    let mut seed: u64 = random();
    println!("test_basic3 seed {}", seed);

    let testsize = 100_000;
    let mut keys: Vec<u64> = Vec::with_capacity(testsize);
    keys.resize(testsize, Default::default());
    for key in keys.iter_mut() {
        *key = splitmix64(&mut seed);
    }

    let filter = {
        let mut filter = Xor8::<RandomState>::new();
        keys.iter().for_each(|key| filter.insert(key));
        filter.build();
        filter
    };

    for key in keys.iter() {
        assert!(filter.contains(key), "key {} not present", key);
    }

    let (falsesize, mut matches) = (10_000_000, 0_f64);
    let bpv = (filter.finger_prints.len() as f64) * 8.0 / (testsize as f64);
    println!("test_basic3 bits per entry {} bits", bpv);
    assert!(bpv < 10.0, "bpv({}) >= 10.0", bpv);

    for _ in 0..falsesize {
        let v = splitmix64(&mut seed);
        if filter.contains(&v) {
            matches += 1_f64;
        }
    }
    let fpp = matches * 100.0 / (falsesize as f64);
    println!("test_basic3 false positive rate {}%", fpp);
    assert!(fpp < 0.40, "fpp({}) >= 0.40", fpp);
}

#[test]
fn test_xor8_basic4() {
    let seed: u128 = random();
    println!("test_basic4 seed {}", seed);
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let testsize = 1_000_000;
    let mut keys: Vec<u64> = Vec::with_capacity(testsize);
    keys.resize(testsize, Default::default());
    for key in keys.iter_mut() {
        *key = rng.gen();
    }

    let filter = {
        let mut filter = Xor8::<BuildHasherDefault>::new();
        filter.populate(&keys);
        filter.build();
        filter
    };

    for key in keys.iter() {
        assert!(filter.contains(key), "key {} not present", key);
    }

    let (falsesize, mut matches) = (10_000_000, 0_f64);
    let bpv = (filter.finger_prints.len() as f64) * 8.0 / (testsize as f64);
    println!("test_basic4 bits per entry {} bits", bpv);
    assert!(bpv < 10.0, "bpv({}) >= 10.0", bpv);

    for _ in 0..falsesize {
        if filter.contains(&rng.gen::<u64>()) {
            matches += 1_f64;
        }
    }
    let fpp = matches * 100.0 / (falsesize as f64);
    println!("test_basic4 false positive rate {}%", fpp);
    assert!(fpp < 0.40, "fpp({}) >= 0.40", fpp);
}

#[test]
fn test_xor8_basic5() {
    let seed: u128 = random();
    println!("test_basic5 seed {}", seed);
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let testsize = 1_000_000;
    let mut keys: Vec<u64> = Vec::with_capacity(testsize);
    keys.resize(testsize, Default::default());
    for key in keys.iter_mut() {
        *key = rng.gen();
    }

    let filter = {
        let mut filter = Xor8::<BuildHasherDefault>::new();
        filter.populate_keys(&keys);
        filter.build();
        filter
    };

    for key in keys.into_iter() {
        assert!(filter.contains_key(key), "key {} not present", key);
    }

    let (falsesize, mut matches) = (10_000_000, 0_f64);
    let bpv = (filter.finger_prints.len() as f64) * 8.0 / (testsize as f64);
    println!("test_basic5 bits per entry {} bits", bpv);
    assert!(bpv < 10.0, "bpv({}) >= 10.0", bpv);

    for _ in 0..falsesize {
        if filter.contains(&rng.gen::<u64>()) {
            matches += 1_f64;
        }
    }
    let fpp = matches * 100.0 / (falsesize as f64);
    println!("test_basic5 false positive rate {}%", fpp);
    assert!(fpp < 0.40, "fpp({}) >= 0.40", fpp);
}

#[test]
fn test_xor8_basic6() {
    let mut seed: u64 = random();
    println!("test_basic6 seed {}", seed);

    let testsize = 100_000;
    let mut keys: Vec<u64> = Vec::with_capacity(testsize);
    keys.resize(testsize, Default::default());
    for key in keys.iter_mut() {
        *key = splitmix64(&mut seed);
    }

    let filter = {
        let mut filter = Xor8::<BuildHasherDefault>::new();
        keys.iter().for_each(|key| filter.insert(key));
        filter.build();
        filter
    };

    for key in keys.iter() {
        assert!(filter.contains(key), "key {} not present", key);
    }

    let (falsesize, mut matches) = (10_000_000, 0_f64);
    let bpv = (filter.finger_prints.len() as f64) * 8.0 / (testsize as f64);
    println!("test_basic6 bits per entry {} bits", bpv);
    assert!(bpv < 10.0, "bpv({}) >= 10.0", bpv);

    for _ in 0..falsesize {
        let v = splitmix64(&mut seed);
        if filter.contains(&v) {
            matches += 1_f64;
        }
    }
    let fpp = matches * 100.0 / (falsesize as f64);
    println!("test_basic6 false positive rate {}%", fpp);
    assert!(fpp < 0.40, "fpp({}) >= 0.40", fpp);
}
