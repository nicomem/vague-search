use super::index::*;
use std::{num::NonZeroU32, ops::Range};

/// A [CompiledTrie](crate::CompiledTrie) node following a Patricia trie structure.
#[derive(Debug, Clone)]
pub struct PatriciaNode {
    /// The number of siblings of the node.
    /// The next sibling is located at the next index in the node array.
    // There are more than u16::MAX characters in unicode, so u32 must be used.
    pub nb_siblings: u32,

    /// The index of the first child in the node array.
    pub index_first_child: IndexNode,

    /// The word frequency. If None, the word does not exist in the dictionary.
    pub word_freq: Option<NonZeroU32>,

    /// The range of characters associated to this node in the characters array.
    pub char_range: Range<IndexChar>,
}

/// A [CompiledTrie](crate::CompiledTrie) node following a naive trie structure.
#[derive(Debug, Clone)]
pub struct NaiveNode {
    /// The number of siblings of the node.
    /// The next sibling is located at the next index in the node array.
    pub nb_siblings: u32,

    /// The index of the first child in the node array.
    pub index_first_child: IndexNode,

    /// The word frequency. If None, the word does not exist in the dictionary.
    pub word_freq: Option<NonZeroU32>,

    /// The character associated to this node.
    pub character: char,
}

#[derive(Debug, Clone)]
pub struct RangeNode {
    /// The number of siblings of the node.
    /// The next sibling is located at the next index in the node array.
    pub nb_siblings: u32,

    /// The first character represented by this node.
    pub first_char: char,

    /// The number of characters represented by this node.
    pub len: u32,

    /// The index of the range in the eponymic array.
    pub index_range: IndexRange,
}

/// A node of a compiled trie.
/// Can be of different structure depending on the situation to optimize
/// memory consumption and execution speed.
#[derive(Debug, Clone)]
pub enum CompiledTrieNode {
    /// Node following the structure of a PATRICIA trie.
    /// More efficient to hold multiple-characters strings
    PatriciaNode(PatriciaNode),

    /// Node following the structure of a naive trie.
    /// More efficient to hold one-character strings.
    NaiveNode(NaiveNode),

    /// Node representing a range of characters where children are stored
    /// in the range array.
    RangeNode(RangeNode),
}

// Implement getters for fields that are contained in all enumeration values.
macro_rules! impl_get_field {
    ($field:ident, $ret:ty) => {
        /// Get the corresponding field of a node.
        pub fn $field(&self) -> $ret {
            match self {
                Self::NaiveNode(node) => node.$field,
                Self::PatriciaNode(node) => node.$field,
                Self::RangeNode(node) => node.$field,
            }
        }
    };
}

impl CompiledTrieNode {
    impl_get_field!(nb_siblings, u32);
}
