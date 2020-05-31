# vague-search

An approximate search engine (project for the text-mining course).

## Authors

- Nicolas Mémeint
- Tom Méchineau

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
