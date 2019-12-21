# xorfilter: Rust library implementing xor filters

Implementation of [Xor Filters: Faster and Smaller Than Bloom and Cuckoo Filters](https://arxiv.org/abs/1912.08258)
in [rust-lang](https://www.rust-lang.org/), Journal of Experimental Algorithmics (to appear).

This package is a port from its [Original implementation](https://github.com/FastFilter/xorfilter)
in golang.

**Open issues**

[ ] Serialize / Deserialize Xor8 type.

**Benchmarks**

Benchmark number for original golang implementation.

```text
BenchmarkPopulate100000-32          2000            695796 ns/op
BenchmarkContains100000-32      200000000                7.03 ns/op
```

Benchmark number for this rust-lang implementation.

```test
test bench_contains_100000 ... bench:           7 ns/iter (+/- 0)
test bench_populate_100000 ... bench:     274,349 ns/iter (+/- 18,650)
```

Measure of _false-positive-rate_ and _bits-per-entry_ in original golang implementation.

```text
bits per entry  9.864
false positive rate  0.3874
```

Measure of _false-positive-rate_ and _bits-per-entry_ in this rust-lang implementation.

```text
bits per entry 9.864 bits
false positive rate 0.3866%
```
