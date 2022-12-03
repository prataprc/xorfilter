use std::sync::Arc;
use std::thread;
use std::time;

use rand::random;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use structopt::StructOpt;
use xorfilter::BuildHasherDefault;
use xorfilter::Fuse16;
use xorfilter::Fuse8;
use xorfilter::Xor8;

/// Command line options.
#[derive(Clone, StructOpt)]
pub struct Opt {
    #[structopt(long = "seed", default_value = "0")]
    seed: u64,

    #[structopt(long = "loads", default_value = "10000000")]
    loads: usize,

    #[structopt(long = "gets", default_value = "10000000")]
    gets: usize,

    #[structopt(long = "readers", default_value = "1")]
    readers: usize,

    command: String,
}

fn main() {
    let mut opts = Opt::from_args();
    if opts.seed == 0 {
        opts.seed = random();
    }

    match opts.command.as_str() {
        "xor8" => run_xor8(opts),
        "fuse8" => run_fuse8(opts),
        "fuse16" => run_fuse16(opts),
        _ => unreachable!(),
    }
}

fn run_xor8(opts: Opt) {
    let keys: Vec<u64> = (0..(opts.loads as u64)).collect();

    let mut filter = Xor8::<BuildHasherDefault>::new();
    filter.populate(&keys);

    let start = time::Instant::now();
    filter.build().unwrap();
    println!("Took {:?} to build {} keys", start.elapsed(), keys.len());

    let mut handles = vec![];
    let keys = Arc::new(keys);
    for j in 0..opts.readers {
        let (opts, filter, keys) = (opts.clone(), filter.clone(), Arc::clone(&keys));
        let handle = thread::spawn(move || {
            let mut rng = StdRng::seed_from_u64(opts.seed);
            let (mut hits, start) = (0, time::Instant::now());
            for _i in 0..opts.gets {
                let off: usize = rng.gen::<usize>() % keys.len();
                if filter.contains(&keys[off]) {
                    hits += 1;
                }
            }
            println!(
                "Reader-{} took {:?} to check {} keys, hits:{} ",
                j,
                start.elapsed(),
                keys.len(),
                hits
            );
        });
        handles.push(handle);
    }

    for handle in handles.into_iter() {
        handle.join().unwrap()
    }
}

fn run_fuse8(opts: Opt) {
    let keys: Vec<u64> = (0..(opts.loads as u64)).collect();

    let mut filter = Fuse8::<BuildHasherDefault>::new(keys.len() as u32);
    filter.populate(&keys);

    let start = time::Instant::now();
    filter.build().unwrap();
    println!("Took {:?} to build {} keys", start.elapsed(), keys.len());

    let mut handles = vec![];
    let keys = Arc::new(keys);
    for j in 0..opts.readers {
        let (opts, filter, keys) = (opts.clone(), filter.clone(), Arc::clone(&keys));
        let handle = thread::spawn(move || {
            let mut rng = StdRng::seed_from_u64(opts.seed);
            let (mut hits, start) = (0, time::Instant::now());
            for _i in 0..opts.gets {
                let off: usize = rng.gen::<usize>() % keys.len();
                if filter.contains(&keys[off]) {
                    hits += 1;
                }
            }
            println!(
                "Reader-{} took {:?} to check {} keys, hits:{} ",
                j,
                start.elapsed(),
                keys.len(),
                hits
            );
        });
        handles.push(handle);
    }

    for handle in handles.into_iter() {
        handle.join().unwrap()
    }
}

fn run_fuse16(opts: Opt) {
    let keys: Vec<u64> = (0..(opts.loads as u64)).collect();

    let mut filter = Fuse16::<BuildHasherDefault>::new(keys.len() as u32);
    filter.populate(&keys);

    let start = time::Instant::now();
    filter.build().unwrap();
    println!("Took {:?} to build {} keys", start.elapsed(), keys.len());

    let mut handles = vec![];
    let keys = Arc::new(keys);
    for j in 0..opts.readers {
        let (opts, filter, keys) = (opts.clone(), filter.clone(), Arc::clone(&keys));
        let handle = thread::spawn(move || {
            let mut rng = StdRng::seed_from_u64(opts.seed);
            let (mut hits, start) = (0, time::Instant::now());
            for _i in 0..opts.gets {
                let off: usize = rng.gen::<usize>() % keys.len();
                if filter.contains(&keys[off]) {
                    hits += 1;
                }
            }
            println!(
                "Reader-{} took {:?} to check {} keys, hits:{} ",
                j,
                start.elapsed(),
                keys.len(),
                hits
            );
        });
        handles.push(handle);
    }

    for handle in handles.into_iter() {
        handle.join().unwrap()
    }
}
