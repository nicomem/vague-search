# vague-search

An approximate search engine (project for the text-mining course).

## Authors

- Nicolas Mémeint
- Tom Méchineau

## Features

- Fast
  - 850k qps at distance 0
  - 4.4k qps at distance 1
  - 120 qps at distance 2
- Low memory footprint
  - 20Mo at distance 4
  - 160Mo at distance 10
- Compatible with **any** valid UTF-8 words
  - Even [emojis](https://en.wikipedia.org/wiki/Emoji) or [diacritics](https://en.wikipedia.org/wiki/Diacritic).
- Optimized for single-core usage

## Pre-requisites

- Rust toolchain >= 1.47
  - See [the Rust website](https://www.rust-lang.org/learn/get-started) for installation instructions
- *optional* A POSIX-compatible OS
  - If your OS is Windows, the entire compiled dictionary will be loaded
   instead of loading it dynamically via the `mmap` system-call

## Usage

An example list of shell commands to build and run the project:

```bash
# Build the binaries in the current folder
./build.sh

# Compile the dictionary
./TextMiningCompiler /path/to/words.txt /path/to/dict.bin

# Search words in the dictionary
echo "approx 0 test" | ./TextMiningApp /path/to/dict.bin
echo "approx 1 test" | ./TextMiningApp /path/to/dict.bin
echo "approx 2 test" | ./TextMiningApp /path/to/dict.bin
echo -e "approx 0 test\napprox 1 test\napprox 2 test\napprox 3 test\napprox 4 test" | ./TextMiningApp /path/to/dict.bin
cat test.txt | ./TextMiningApp /path/to/dict.bin
```

## Documentation

```bash
cargo doc --workspace --open
```

## Tests

```bash
cargo test --workspace
```

## Questions for the projet

See the [QUESTIONS.md](./QUESTIONS.md) for the answers to the Text-Mining course questions.
