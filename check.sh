#! /usr/bin/env bash

cargo +nightly build
cargo +stable build

cargo +nightly doc
cargo +stable doc

cargo +nightly test;
cargo +stable test

cargo +nightly bench;
cargo +stable test

cargo +nightly clippy --all-targets --all-features

