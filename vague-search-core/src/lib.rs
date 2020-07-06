//! The core library for the vague-search binaries.
//!
//! Define shared data structures and functions to be used in both binaries.

mod dictionary_file;
mod error;
mod trie;
mod utils;

pub use dictionary_file::*;
pub use error::{Error, Result};
pub use trie::{compiled_trie::*, index::*, trie_node::*, trie_node_interface::*};
