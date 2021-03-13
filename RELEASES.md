0.3.0
=====

* Xor8 to bytes and vice-versa conversion.
* implement Default trait.
* implement mkit's IntoCbor and FromCbor traits for Cbor serialization.
* improve test cases.
* use criterion for benchmark.
* clippy fixes.
* ci scripts.

0.2.0
=====

* `write_file()` and `read_file()` methods on Xor8 type will take
  `&ffi::OsStr` instead of `&str`. This more consistent with rust-idiom.
* cleanup test-cases.
* cleanup Makefile.

0.1.0
=====

* First release

Refer to [release-checklist][release-checklist].

[release-checklist]: https://prataprc.github.io/rust-crates-release-checklist.html
