[![Rustdoc](https://img.shields.io/badge/rustdoc-hosted-blue.svg)](https://docs.rs/xorfilter-rs)
[![simple-build-test](https://github.com/bnclabs/xorfilter/actions/workflows/simple-build-test.yml/badge.svg)](https://github.com/bnclabs/xorfilter/actions/workflows/simple-build-test.yml)

Rust library implementing xor filters
-------------------------------------

Implementation of [Xor Filters: Faster and Smaller Than Bloom and Cuckoo Filters](https://arxiv.org/abs/1912.08258)
in [rust-lang](https://www.rust-lang.org/), Journal of Experimental Algorithmics (to appear).

This package is a port from its [golang implementation](https://github.com/FastFilter/xorfilter).

### How to use _xorfilter_ in my rust project ?

Add the following under project's `Cargo.toml`:

```toml
[dependencies]
xorfilter-rs = "0.2.0"
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

Open issues
-----------

* [ ] Serialize / Deserialize Xor8 type.
* [ ] Incrementally adding keys to a pre-built Xor8 instance.
* [ ] Gather benchmark results for other implementations - Go, C, C++, Erlang, Java, Python.

Benchmarks
----------

Following are the results for a set of 10-million `u64` keys:

|             |  build 10M keys |  membership |   FPP   |  Bits/Entry |
|-------------|-----------------|-------------|---------|-------------|
| Xor8-C      |   1.206 secs    |    NA       | 0.389 % |  9.84 bits  |
| Xor8-rust   |   1.809 secs    | 61.716 ns   | 0.392 % |  9.84 bits  |
| Fuse8-C     |   0.508 secs    |    NA       | 0.390 % |  9.02 bits  |
| Fuse8-rust  |   0.577 secs    | 42.657 ns   | 0.392 % |  9.02 bits  |
| Fuse16-C    |   0.515 secs    |    NA       | 0.001 % | 18.04 bits  |
| Fuse16-rust |   0.621 secs    | 54.657 ns   | 0.001 % | 18.03 bits  |

* **Build time** is measured in `Seconds`, for 10 million entries.
* **Membership** is measured in `Nanosec`, for single lookup in a set of 10 million entries.
* **FPP** = False Positive Probability measured in percentage

Useful links
------------

* [Xor Filters: Faster and Smaller Than Bloom and Cuckoo Filters](https://arxiv.org/abs/1912.08258)
* [Blog post by Daniel Lemire](https://lemire.me/blog/2019/12/19/xor-filters-faster-and-smaller-than-bloom-filters/)


Contribution
------------

* Simple workflow. Fork - Modify - Pull request.
* Before creating a PR,
  * Run `make build` to confirm all versions of build is passing with
    0 warnings and 0 errors.
  * Run `check.sh` with 0 warnings, 0 errors and all test-cases passing.
  * Run `perf.sh` with 0 warnings, 0 errors and all test-cases passing.
  * [Install][spellcheck] and run `cargo spellcheck` to remove common spelling mistakes.
* [Developer certificate of origin][dco] is preferred.

[dco]: https://developercertificate.org/
[spellcheck]: https://github.com/drahnr/cargo-spellcheck
