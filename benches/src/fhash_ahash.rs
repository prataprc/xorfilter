use std::collections::{HashMap, hash_map::RandomState};
use std::hash::BuildHasher;

fn test_with<S: BuildHasher + Default>() {
    for _ in 0..10 {
        let mut map: HashMap<i32,i32,S> = HashMap::with_capacity_and_hasher(1000_0000, Default::default());
        let now = std::time::Instant::now();
        for key in 0..1000_0000 {
            map.insert(key, key);
        }
        let elapsed = now.elapsed();
        println!("{:?}", elapsed);
    }
}

fn main() {
    println!("std:");
    test_with::<RandomState>();
    println!("ahash:");
    test_with::<ahash::RandomState>();
    println!("fxhash:");
    test_with::<fxhash::FxBuildHasher>()
}