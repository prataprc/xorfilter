use super::*;
use rand::{prelude::random, rngs::SmallRng, Rng, SeedableRng};

#[test]
fn test_fuse8() {
    let seed: u128 = random();
    println!("test_fuse8 seed {}", seed);
    let mut rng = SmallRng::from_seed(seed.to_le_bytes());

    for size in (10000000_u32..10000001).map(|x| x * 10) {
        let mut filter = Fuse8::new(size).unwrap();
        // we need some set of values
        let big_set: Vec<u64> = (0_u64..(size as u64)).collect();
        // we construct the filter
        filter.populate(&big_set);
        for key in big_set.iter() {
            assert!(filter.contains(*key), "expected positive for key: {}", key);
        }

        let mut random_matches = 0_usize;
        let trials = 10000000_usize; //(uint64_t)rand() << 32 + rand()
        for _i in 0..trials {
            let random_key: u64 = rng.gen();
            if filter.contains(random_key) && (random_key >= (size as u64)) {
                random_matches += 1;
            }
        }

        println!("For size {}", size);
        let fpp: f64 = (random_matches as f64) / (trials as f64);
        println!(" fpp {:3.5} (estimated)", fpp);
        let bpe: f64 = ((filter.size_of() as f64) * 8.0) / (size as f64);
        println!(" bits per entry {:3.2}", bpe);
        println!(
            " bits per entry {:3.2} (theoretical lower bound)",
            -fpp.ln() / 2.0_f64.ln(),
        );
        println!(" efficiency ratio {:3.3}", bpe / (-fpp.ln() / 2.0_f64.ln()));
    }
}
