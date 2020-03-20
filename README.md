![](https://github.com/bnclabs/xorfilter/workflows/simple-build-test/badge.svg)

# Rust library implementing xor filters

Implementation of [Xor Filters: Faster and Smaller Than Bloom and Cuckoo Filters](https://arxiv.org/abs/1912.08258)
in [rust-lang](https://www.rust-lang.org/), Journal of Experimental Algorithmics (to appear).

This package is a port from its [golang implementation](https://github.com/FastFilter/xorfilter).

### How to use _xorfilter_ in my rust project ?

Add the following under project's `Cargo.toml`:

```toml
[dependencies]
xorfilter-rs = 0.2.0
```

or

```toml
[dependencies]
xorfilter-rs = { git = "https://github.com/bnclabs/xorfilter" }
```

```rust
use xorfilter::Xor8;

let mut keys: Vec<u64> = vec![];
for _ in 0..num_keys {
    keys.push(rng.gen());
}

let mut filter = Xor8::new(); // new filter.
filter.populate_keys(&keys); // populate keys.
filter.build(); // build bitmap.

for key in 0..lookup {
    // there can be false positives, but no false negatives.
    filter.contains_key(key);
}
```

### Open issues

* [ ] Serialize / Deserialize Xor8 type.
* [ ] Incrementally adding keys to a pre-built Xor8 instance.

### Benchmarks

Benchmark number for original golang implementation.

```text
BenchmarkPopulate100000-32          2000            695796 ns/op
BenchmarkContains100000-32      200000000                7.03 ns/op
```

Benchmark number for this rust-lang implementation.

```test
test bench_populate_100000 ... bench:     274,349 ns/iter (+/- 18,650)
test bench_contains_100000 ... bench:           7 ns/iter (+/- 0)
```

Measure of _false-positive-rate_ and _bits-per-entry_ in
original golang implementation, using random set of keys.

```text
bits per entry  9.864
false positive rate  0.3874
```

Measure of _false-positive-rate_ and _bits-per-entry_ in
this rust-lang implementation, using random set of keys.

```text
bits per entry 9.864 bits
false positive rate 0.3866%
```

### Resources

* [Xor Filters: Faster and Smaller Than Bloom and Cuckoo Filters](https://arxiv.org/abs/1912.08258)
* [Blog post by Daniel Lemire](https://lemire.me/blog/2019/12/19/xor-filters-faster-and-smaller-than-bloom-filters/)
