#!/usr/bin/env bash

set -ex

: ${TARGET?"The TARGET environment variable must be set."}

# Tests are all super fast anyway, and they fault often enough on travis that
# having only one thread increases debuggability to be worth it.
export RUST_TEST_THREADS=1
export RUST_BACKTRACE=full
export RUST_TEST_NOCAPTURE=1

# The source directory is read-only. Need to copy internal crates to the target
# directory for their Cargo.lock to be properly written.
mkdir target || true

rustc --version
cargo --version
echo "TARGET=${TARGET}"
echo "HOST=${HOST}"
echo "RUSTFLAGS=${RUSTFLAGS}"
echo "CARGO_SUBCMD=${CARGO_SUBCMD}"
echo "CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS}"
echo "CARGO_INCREMENTAL=${CARGO_INCREMENTAL}"
echo "RUST_TEST_THREADS=${RUST_TEST_THREADS}"
echo "RUST_BACKTRACE=${RUST_BACKTRACE}"
echo "RUST_TEST_NOCAPTURE=${RUST_TEST_NOCAPTURE}"

cargo test
cargo test --release
