//! The core library for the vague-search binaries.
//!
//! Define shared data structures and functions to be used in both binaries.

mod compiled_trie;
mod dictionary_file;
mod error;
mod utils;

pub use compiled_trie::{CompiledTrie, CompiledTrieNode, NaiveNode, PatriciaNode};
pub use dictionary_file::{DictionaryFile, Header};
pub use error::{Error, Result};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
