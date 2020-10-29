use rand::{prelude::random, rngs::SmallRng, Rng, SeedableRng};
use std::collections::hash_map::RandomState;
use xorfilter::Xor8;

/// Generate a filter with random keys
fn generate_filter() -> Xor8<RandomState> {
    let seed: u128 = random();
    println!("seed {}", seed);
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    let testsize = 10000;
    let mut keys: Vec<u64> = Vec::with_capacity(testsize);
    keys.resize(testsize, Default::default());
    for key in keys.iter_mut() {
        *key = rng.gen();
    }
    let mut filter = Xor8::<RandomState>::new();
    filter.populate(&keys);
    filter.build();
    filter
}

#[test]
fn test_same_filter_encode_decode() {
    let file_path = {
        let mut fpath = std::env::temp_dir();
        fpath.push("xorfilter-test-same-filter-encode-decode");
        fpath.into_os_string()
    };
    let filter = generate_filter();

    filter
        .write_file(&file_path)
        .unwrap_or_else(|err| panic!("Write to {:?} failed {}", file_path, err));
    let filter_read = Xor8::read_file(&file_path)
        .unwrap_or_else(|err| panic!("Read from {:?} failed {}", file_path, err));
    assert!(
        filter_read == filter,
        "Filter unequals after encode and decode"
    );

    let filter_second = generate_filter();
    assert!(
        filter_read != filter_second,
        "Random generated filters should not be the same"
    );
}

#[test]
fn test_same_filter_bytes_encoding() {
    let filter = generate_filter();

    let buf = filter.to_bytes();
    let filter_read =
        Xor8::from_bytes(buf).unwrap_or_else(|err| panic!("Read from bytes failed {}", err));
    assert!(
        filter_read == filter,
        "Filter unequals after encode and decode"
    );

    let filter_second = generate_filter();
    assert!(
        filter_read != filter_second,
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
        "learning curves are a blessing in disguise",
    ];
    let hash_builder = RandomState::new();
    let mut filter = Xor8::with_hasher(hash_builder);
    filter.populate(&rust_tips);
    filter.build();

    // Test all keys(rust_tips)
    for tip in rust_tips {
        assert!(filter.contains(tip));
    }
    // Remove last one character
    assert!(!filter.contains("show up with cod"));
    // String not in keys(rust_tips)
    assert!(!filter.contains("No magic, just code"));
}
