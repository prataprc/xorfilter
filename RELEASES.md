0.6.0
=====

* Added len() method for Fuse8
* Added len() method for Fuse16

0.5.1
=====

* Fuse8: handle duplicates without sorting.
* Fuse8: improve test case with duplicate keys.
* CI: improvement to check.sh script.
* rustdoc.

0.5.0
=====

**Breaking Change**

File version moves from `TL1` to `TL2`.
  * Now includes `hash_builder` field as part of Xor8 serialization.
  * Test cases for TL1 (backward compatibility) and TL2.
  * METADATA includes length of the serialized `hash_builder`.
  * Shape of the serialized file has changed.
  * `Xor8::write_file`, `Xor8::read_file`, `Xor8::to_bytes`, `Xor8::from_bytes`
    methods expect that type parameter implements `Default`, `Clone`,
    `From<Vec<u8>>`, `Into<Vec<u8>>` traits.

* Bugfix: Check for duplicate key-digest. It is possible that, when using
  `insert()`, `populate()`, keys could generate duplicate digest. This will
  lead to failure while building the filter. To mitigate this issue we are
  maintaining the digests in sort order.
* `Fuse8` and `Fuse16` implementation.
* hasher: NoHash, to use the types and its methods using `u64` digests.
* Add `size_of()` method to filter types.
* Support key-set of size 0, 1, and 2.
* Improve test cases.
* rustdoc
* cargo: fix category slug

0.4.0
=====

* package maintanence.

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
