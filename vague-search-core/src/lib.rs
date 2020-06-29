//! The core library for the vague-search binaries.
//!
//! Define shared data structures and functions to be used in both binaries.

mod compiled_trie;
mod dictionary_file;
mod error;
mod trie;
mod utils;

pub use compiled_trie::{
    compiled_trie::CompiledTrie,
    index::*,
    trie_node::{CompiledTrieNode, NaiveNode, PatriciaNode, RangeNode},
};
pub use dictionary_file::{DictionaryFile, Header};
pub use error::{Error, Result};
