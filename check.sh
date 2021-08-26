#! /usr/bin/env bash

export RUST_BACKTRACE=full
export RUSTFLAGS=-g
exec > check.out
exec 2>&1

set -o xtrace

date
exec_prg() {

    for i in {0..5};
    do
        cargo +nightly test --release -- --nocapture || exit $?;
        cargo +nightly test -- --nocapture || exit $?;
        cargo +stable test --release -- --nocapture || exit $?;
        cargo +stable test -- --nocapture || exit $?;
    done
}

exec_prg
