#! /usr/bin/env bash

exec > $1
exec 2>&1

set -o xtrace

PERF=$HOME/.cargo/target/release/perf

date; time cargo +nightly bench -- --nocapture || exit $?

date; time cargo +nightly run --release --bin perf --features=perf -- --loads 10000000 --gets 10000000 xor8 || exit $?
date; time cargo +nightly run --release --bin perf --features=perf -- --loads 10000000 --gets 10000000 fuse8 || exit $?
date; time cargo +nightly run --release --bin perf --features=perf -- --loads 10000000 --gets 10000000 fuse16 || exit $?
