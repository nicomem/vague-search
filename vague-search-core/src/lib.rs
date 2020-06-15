mod compiled_trie;
mod dictionary_file;
mod error;
mod utils;

pub use compiled_trie::{CompiledTrie, CompiledTrieNode};
pub use dictionary_file::{DictionaryFile, Header};
pub use error::{Error, Result};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
