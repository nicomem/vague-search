//! The core library for the vague-search binaries.
//!
//! Define shared data structures and functions to be used in both binaries.

mod dictionary_file;
mod error;
mod trie;
mod utils;

pub use dictionary_file::{DictionaryFile, Header};
pub use error::{Error, Result};
pub use trie::{
    compiled_trie::{CompiledTrie, IndexNode},
    trie_node::{CompiledTrieNode, NaiveNode, PatriciaNode},
};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
