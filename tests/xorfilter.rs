use std::ffi;

use rand::prelude::random;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use xorfilter::xor8::Xor8;
use xorfilter::xor8::Xor8Builder;
use xorfilter::BuildHasherDefault;

#[test]
fn test_same_filter_encode_decode() {
    let seed: u64 = random();
    println!("test_same_filter_encode_decode seed:{}", seed);

    let file_path = {
        let mut fpath = std::env::temp_dir();
        fpath.push("xorfilter-test-same-filter-encode-decode");
        fpath.into_os_string()
    };
    let filter = generate_filter(seed);

    filter.write_file(&file_path).expect("fail write_file");
    let filter_read = Xor8::read_file(&file_path).expect("fail read_file");
    assert!(
        filter_read == filter,
        "Filter unequals after encode and decode"
    );

    let filter_second = generate_filter(seed + 1000);
    assert!(
        filter_read != filter_second,
        "Random generated filters should not be the same"
    );
}

#[test]
fn test_same_filter_bytes_encoding_tl1() {
    use std::path;

    let keys: Vec<u32> = (1..10000).map(|i| (i * 2) + 1).collect();
    let missing: Vec<u32> = (1..20).map(|i| (i * 2)).collect();

    let file_path = {
        let mut loc = path::PathBuf::new();
        loc.push(path::Path::new(file!()).parent().unwrap().to_str().unwrap());
        loc.push("tl1-serialized.data");
        loc.into_os_string()
    };

    // save_file(file_path.clone(), &keys);

    let filter = Xor8::<BuildHasherDefault>::read_file(&file_path)
        .expect("Read from bytes failed");

    for key in keys.iter() {
        assert!(filter.contains(key))
    }

    for key in missing.iter() {
        assert!(!filter.contains(key))
    }
}

#[test]
fn test_same_filter_bytes_encoding_tl2() {
    let seed: u64 = random();
    println!("test_same_filter_bytes_encoding_tl1 seed:{}", seed);

    let filter = generate_filter(seed);

    let buf = filter.to_bytes();
    let filter_read = Xor8::from_bytes(buf).expect("Read from bytes failed");
    assert!(
        filter_read == filter,
        "Filter unequals after encode and decode"
    );

    let filter_second = generate_filter(seed + 1000);
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
    let hash_builder = BuildHasherDefault::default();
    let mut builder = Xor8Builder::with_hasher(hash_builder);
    builder.populate(&rust_tips);
    let filter = builder.build().expect("build failed");

    // Test all keys(rust_tips)
    for tip in rust_tips {
        assert!(filter.contains(tip));
    }
    // Remove last one character
    assert!(!filter.contains("show up with cod"));
    // String not in keys(rust_tips)
    assert!(!filter.contains("No magic, just code"));
}

/// Generate a filter with random keys
fn generate_filter(seed: u64) -> Xor8<BuildHasherDefault> {
    let mut rng = StdRng::seed_from_u64(seed);

    let testsize = 10000;
    let mut keys: Vec<u64> = Vec::with_capacity(testsize);
    keys.resize(testsize, u64::default());
    for key in keys.iter_mut() {
        *key = rng.gen();
    }

    let mut builder = Xor8Builder::<BuildHasherDefault>::new();
    builder.populate(&keys);
    builder.build().expect("build failed")
}

// hack to generate tl1 serialized Xor8 instance.
#[allow(dead_code)]
fn save_file(file_path: ffi::OsString, keys: &[u32]) {
    let mut builder = Xor8Builder::<BuildHasherDefault>::new();
    builder.populate(keys);
    let filter = builder.build().expect("build failed");
    filter.write_file(&file_path).expect("error saving tl1 to file");
}
