0.1.0
=====

Code Review checklist
=====================

* [ ] Check and confirm dead-code.
* [ ] Check and confirm ignored test cases.
* [ ] Replace panic!(), assert!(), unreachable!(), unimplemented!(),
      macros with Err(Error).
* [ ] Avoid println!() macro in production code.
* [ ] Validate the usage of:
    * [ ] unwrap() calls.
    * [ ] ok() calls on Result/Option types.
    * [ ] unsafe { .. } blocks.
* [ ] Trim trait constraits for exported types, exported functions
  and methods.

Release Checklist
=================

* Bump up the version:
  * __major__: backward incompatible API changes.
  * __minor__: backward compatible API Changes.
  * __patch__: bug fixes.
* Travis-CI integration.
* Cargo checklist
  * cargo +stable build; cargo +nightly build
  * cargo +stable doc
  * cargo +nightly clippy --all-targets --all-features
  * cargo +nightly test; cargo +stable test
  * cargo +nightly bench;
  * cargo fix --edition --all-targets
* Create a git-tag for the new version.
* Cargo publish the new version.
* Badges
  * Build passing, Travis continuous integration.
  * Code coverage, codecov and coveralls.
  * Crates badge
  * Downloads badge
  * License badge
  * Rust version badge.
  * Maintenance-related badges based on isitmaintained.com
  * Documentation
  * Gitpitch
* Targets
  * RHEL
  * SUSE
  * Debian
  * Centos
  * Ubuntu
  * Mac-OS
  * Windows
  * amazon-aws
  * Raspberry-pi
