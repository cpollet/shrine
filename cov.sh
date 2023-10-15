#!/usr/bin/env bash

# https://lib.rs/crates/cargo-llvm-cov

source <(cargo llvm-cov show-env --export-prefix)

if rustup show active-toolchain | grep nightly; then
  export RUSTDOCFLAGS="-C instrument-coverage -Z unstable-options --persist-doctests target/debug/doctestbins"
fi

cargo llvm-cov clean --workspace

cargo build
cargo test

if rustup show active-toolchain | grep nightly; then
  for file in target/debug/doctestbins/*/rust_out; do
    [[ -x $file ]] && $file
  done
fi

cargo llvm-cov report --html --hide-instantiations
