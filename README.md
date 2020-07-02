# vague-search

An approximate search engine (project for the text-mining course).

## Authors

- Nicolas Mémeint
- Tom Méchineau

## Features

- Fast
- Low memory footprint
- Compatible with **any** valid UTF-8 words

## Pre-requisites

- Latest (stable) Rust toolchain
  - See [the Rust website](https://www.rust-lang.org/learn/get-started)
- *optional* A POSIX-compatible OS
  - If your OS is Windows or another non-POSIX OS, the entire compiled dictionary will be loaded,
   instead of loading it dynamically via the `mmap` syscall

## Usage

An example list of shell commands to build and run the project:

```shell
# Build the binaries in the current folder
./build.sh

# Compile the dictionary
./TextMiningCompiler /path/to/words.txt /path/to/dict.bin

# Search words in the dictionary
echo "approx 0 test" | ./TextMiningApp /path/to/dict.bin
echo "approx 1 test" | ./TextMiningApp /path/to/dict.bin
echo "approx 2 test" | ./TextMiningApp /path/to/dict.bin
echo "approx 0 test\napprox 1 test\napprox 2 test\napprox 3 test\napprox 4 test" | ./TextMiningApp /path/to/dict.bin
cat test.txt | ./TextMiningApp /path/to/dict.bin
```

## Documentation

```shell
cargo doc --workspace --open
```

## Tests

```shell
cargo test --workspace
```
