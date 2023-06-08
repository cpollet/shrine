#!/usr/bin/env bash

# https://lib.rs/crates/cargo-llvm-cov

source <(cargo llvm-cov show-env --export-prefix)
export RUSTDOCFLAGS="-C instrument-coverage -Z unstable-options --persist-doctests target/debug/doctestbins"
cargo llvm-cov clean --workspace

cargo build
cargo test

for file in target/debug/doctestbins/*/rust_out; do
  [[ -x $file ]] && $file
done

./smoke-tests.sh

cargo llvm-cov report --html
