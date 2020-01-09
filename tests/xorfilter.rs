use std::fs;

use rand::{prelude::random, rngs::SmallRng, Rng, SeedableRng};
use xorfilter::Xor8;

/// Generate a filter with random keys
fn generate_filter() -> Xor8 {
    let seed: u128 = random();
    println!("seed {}", seed);
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let testsize = 10000;
    let mut keys: Vec<u64> = Vec::with_capacity(testsize);
    keys.resize(testsize, Default::default());
    for i in 0..keys.len() {
        keys[i] = rng.gen();
    }
    Xor8::new(&keys)
}

struct TestFile(String);

impl Drop for TestFile {
    fn drop(&mut self) {
        fs::remove_file(&self.0).ok();
    }
}

#[test]
fn test_same_filter_encode_decode() {
    let file_path = TestFile("test_encode.bin".to_string());
    let filter = generate_filter();

    filter
        .write_file(&file_path.0)
        .expect(&format!("Write to {} failed", file_path.0));
    let filter_read =
        Xor8::read_file(&file_path.0).expect(&format!("Read from {} failed", file_path.0));
    assert_eq!(
        filter_read, filter,
        "Filter unequals after encode and decode"
    );

    let filter_second = generate_filter();
    assert_ne!(
        filter_read, filter_second,
        "Random generated filters should not be the same"
    );
}
