#!/bin/sh

# Build all binaries with many optimization flags
RUSTFLAGS="-C target-cpu=native" cargo build --release --workspace

# Copy the result binaries in the root folder with the wanted names
cp target/release/vague-search-index TextMiningCompiler
cp target/release/vague-search TextMiningApp
