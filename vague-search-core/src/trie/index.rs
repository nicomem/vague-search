//! Define index wrappers that can only be used to access their corresponding array.
//! If instead an index was returned as an integer, it could be used with any of
//! the array in the trie.
//! Here, we only implement indexing for the corresponding type of slice and the
//! inner index integer is kept private, to keep everything safe.

use crate::CompiledTrieNode;
use std::{num::NonZeroU32, ops::Index};

/// An element of the range array, accessible via a [RangeNode](CompiledTrieNode::RangeNode).
#[derive(Debug, Clone)]
pub struct RangeElement {
    /// The index of the first child in the node array.
    pub index_first_child: IndexNode,

    /// The word frequency. If None, the word does not exist in the dictionary.
    pub word_freq: Option<NonZeroU32>,
}

/// The node slice stored in the trie.
pub type NodeSlice = [CompiledTrieNode];

/// The character slice stored in the trie.
pub type CharSlice = [char];

/// The range slice stored in the trie.
pub type RangeSlice = [RangeElement];

// Macro to implement slice indexing for corresponding index wrappers
macro_rules! index_wrapper {
    ($index:ident, $elem:ty) => {
        /// Represent a valid index in the [CompiledTrie](crate::CompiledTrie) corresponding array.
        #[derive(Debug, Copy, Clone)]
        pub struct $index {
            index: u32,
        }

        impl Index<$index> for [$elem] {
            type Output = $elem;

            fn index(&self, index: $index) -> &Self::Output {
                &self[index.index as usize]
            }
        }
    };
}

index_wrapper!(IndexNode, CompiledTrieNode);
index_wrapper!(IndexChar, char);
index_wrapper!(IndexRange, RangeElement);

// Implement some useful methods for CompiledTrie.
impl IndexNode {
    pub(super) const fn zero() -> Self {
        Self { index: 0 }
    }

    pub(super) const fn offset_unchecked(self, offset: u32) -> Self {
        Self {
            index: self.index + offset,
        }
    }
}
