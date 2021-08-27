#! /usr/bin/env bash

exec > $1
exec 2>&1

set -o xtrace

PERF=$HOME/.cargo/target/release/perf

date; time cargo +nightly bench -- --nocapture || exit $?
