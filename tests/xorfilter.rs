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

#[test]
fn test_string_keys() {
    // Rust tips: https://ashleygwilliams.github.io/gotober-2018/#103
    let rust_tips = vec![
        "don't rewrite your software in rust",
        "show up with code",
        "don't sell",
        "sell sell sell",
        "the hard part of programming is not programming",
        "the hard part of programming is programming",
        "be prepared for change",
        "be prepared for things to stay the same",
        "have a problem to solve",
        "learning curves are a blessing in disguise",];
    let xor8 = Xor8::new_hashable(&rust_tips);

    // Test all keys(rust_tips)
    for tip in rust_tips {
        assert!(xor8.contains_hashable(tip));
    }
    // Remove last one character
    assert!(!xor8.contains_hashable("show up with cod"));
    // String not in keys(rust_tips)
    assert!(!xor8.contains_hashable("No magic, just code"));
}
