//! The application binary of the vague-search project.
//!
//! Listen for actions in [the standard input stream](std::io::stdin)
//! of the syntax `approx <N> <WORD>` to search for words in a
//! [distance](https://en.wikipedia.org/wiki/Damerau%E2%80%93Levenshtein_distance)
//! of at most N inside a compiled dictionary.
//!
//! See the [vague-search-index](../vague_search_index/index.html) crate for
//! documentation about the dictionary compiler binary.
//!
//! See the [vague-search-core](../vague_search_core/index.html) crate for
//! documentation about types and functions shared by the binaries.

fn main() {
    println!("Hello, World!")
}
