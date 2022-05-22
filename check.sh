#! /usr/bin/env bash

export RUST_BACKTRACE=full
export RUSTFLAGS=-g

exec > $1
exec 2>&1

set -o xtrace

exec_prg() {

    for i in {0..5};
    do
        date; cargo +nightly test --release -- --nocapture || exit $?;
        date; cargo +nightly test -- --nocapture || exit $?;
        date; cargo +nightly test --release --features cbordata -- --nocapture || exit $?;
        date; cargo +nightly test --features cbordata -- --nocapture || exit $?;
        date; cargo +stable test --release -- --nocapture || exit $?;
        date; cargo +stable test -- --nocapture || exit $?;
        date; cargo +stable test --release --features cbordata -- --nocapture || exit $?;
        date; cargo +stable test --features cbordata -- --nocapture || exit $?;
    done
}

exec_prg
