#!/usr/bin/env bash

# https://doc.rust-lang.org/rustc/instrument-coverage.html
# https://github.com/mozilla/grcov
# https://blog.rng0.io/how-to-do-code-coverage-in-rust

rm -rf target/cov
rm -rf target/coverage
mkdir -p target/cov

export CARGO_INCREMENTAL=0
export RUSTFLAGS='-C instrument-coverage'
export LLVM_PROFILE_FILE='target/cov/default_%p-%m.profraw'
cargo test

grcov target/cov \
  --binary-path ./target/debug/deps/ \
  --source-dir . \
  --output-types html \
  --branch \
  --ignore-not-existing \
  --ignore '../*' \
  --ignore "/*" \
  --output-path target/coverage/html
