use super::index::*;
use std::{num::NonZeroU32, ops::Range};

/// A [CompiledTrie](crate::CompiledTrie) node following a Patricia trie structure.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PatriciaNode {
    /// The number of siblings of the node.
    /// The next sibling is located at the next index in the node array.
    // There are more than u16::MAX characters in unicode, so u32 must be used.
    pub nb_siblings: u32,

    /// The index of the first child in the node array.
    pub index_first_child: Option<IndexNodeNonZero>,

    /// The word frequency. If None, the word does not exist in the dictionary.
    pub word_freq: Option<NonZeroU32>,

    /// The range of characters associated to this node in the characters array.
    pub char_range: Range<IndexChar>,
}

/// A [CompiledTrie](crate::CompiledTrie) node following a naive trie structure.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NaiveNode {
    /// The number of siblings of the node.
    /// The next sibling is located at the next index in the node array.
    pub nb_siblings: u32,

    /// The index of the first child in the node array.
    pub index_first_child: Option<IndexNodeNonZero>,

    /// The word frequency. If None, the word does not exist in the dictionary.
    pub word_freq: Option<NonZeroU32>,

    /// The character associated to this node.
    pub character: char,
}

/// A [CompiledTrie](crate::CompiledTrie) node representing a range of characters.
/// This node only represents the range of characters, to access its children,
/// check the [RangeSlice](crate::RangeSlice) of the [CompiledTrie](crate::CompiledTrie).
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RangeNode {
    /// The number of siblings of the node.
    /// The next sibling is located at the next index in the node array.
    pub nb_siblings: u32,

    /// The first character represented by this node.
    pub first_char: char,

    /// The index of the range in the eponymic array.
    pub range: Range<IndexRange>,
}

/// An element of the range array, accessible via a [RangeNode](crate::RangeNode).
/// Since `index_first_child` cannot have the value 0, the struct can be contained
/// inside an Option without using more memory.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct RangeElement {
    /// The index of the first child in the node array.
    /// This index could not be equal to 0 because the 0th node is the trie root,
    /// which is a child to none.
    pub index_first_child: Option<IndexNodeNonZero>,

    /// The word frequency. If None, the word does not exist in the dictionary.
    pub word_freq: Option<NonZeroU32>,
}

/// A node of a compiled trie.
/// Can be of different structure depending on the situation to optimize
/// memory consumption and execution speed.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CompiledTrieNode {
    /// Node following the structure of a PATRICIA trie.
    /// More efficient to hold multiple-characters strings (e.g. bar-foo).
    PatriciaNode(PatriciaNode),

    /// Node following the structure of a naive trie.
    /// More efficient to hold one-character strings (e.g. a-f-i-z).
    NaiveNode(NaiveNode),

    /// Node representing a range of characters where children are stored
    /// in the range array.
    /// More efficient for continuous range of 1-character nodes (e.g. a-b-c-d).
    RangeNode(RangeNode),
}

// Implement getters for fields that are contained in all enumeration values.
macro_rules! impl_get_field {
    ($field:ident, $ret:ty) => {
        /// Get the corresponding field of a node.
        pub fn $field(&self) -> $ret {
            match self {
                Self::NaiveNode(node) => node.$field as $ret,
                Self::PatriciaNode(node) => node.$field as $ret,
                Self::RangeNode(node) => node.$field as $ret,
            }
        }
    };
}

impl CompiledTrieNode {
    impl_get_field!(nb_siblings, u32);
}
