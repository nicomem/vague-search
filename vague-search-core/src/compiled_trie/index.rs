//! Define index wrappers that can only be used to access their corresponding array.
//! If instead an index was returned as an integer, it could be used with any of
//! the array in the trie.
//! Here, we only implement indexing for the corresponding type of slice and the
//! inner index integer is kept private, to keep everything safe.

use std::{num::NonZeroU32, ops::Deref};

/// An element of the range array, accessible via a [RangeNode](crate::RangeNode).
#[derive(Debug, Clone)]
pub struct RangeElement {
    /// The index of the first child in the node array.
    pub index_first_child: IndexNode,

    /// The word frequency. If None, the word does not exist in the dictionary.
    pub word_freq: Option<NonZeroU32>,
}

// Macro to implement slice indexing for corresponding index wrappers
macro_rules! index_wrapper {
    ($index:ident) => {
        /// Represent a valid index in the [CompiledTrie](crate::CompiledTrie) corresponding array.
        #[derive(Debug, Copy, Clone)]
        pub struct $index {
            index: u32,
        }

        impl Deref for $index {
            type Target = u32;

            fn deref(&self) -> &Self::Target {
                &self.index
            }
        }
    };
}

macro_rules! derive_new {
    ($index:ident) => {
        impl $index {
            pub(super) const fn new(index: u32) -> Self {
                Self { index }
            }
        }
    };
}

index_wrapper!(IndexNode);
index_wrapper!(IndexChar);
index_wrapper!(IndexRange);
derive_new!(IndexNode);
derive_new!(IndexChar);
