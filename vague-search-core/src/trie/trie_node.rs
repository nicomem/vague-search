use std::{num::NonZeroU32, ops::Range};

/// **TODO: Check size + reduce fields size**
///
/// A [CompiledTrie](crate::CompiledTrie) node following a Patricia trie structure.
#[derive(Debug, Clone)]
pub struct PatriciaNode {
    /// The number of siblings of the node.
    /// The next sibling is located at the next index in the node array.
    // There are more than u16::MAX characters in unicode, so u32 must be used.
    pub nb_siblings: u32,

    /// The index of the first child in the node array.
    pub index_first_child: u32,

    /// The range of characters associated to this node in the characters array.
    pub char_range: Range<u32>,

    /// The word frequency. If None, the word does not exist in the dictionary.
    pub word_freq: Option<NonZeroU32>,
}

/// **TODO: Check size + reduce fields size**
///
/// A [CompiledTrie](crate::CompiledTrie) node following a naive trie structure.
#[derive(Debug, Clone)]
pub struct NaiveNode {
    /// The number of siblings of the node.
    /// The next sibling is located at the next index in the node array.
    pub nb_siblings: u32,

    /// The index of the first child in the node array.
    pub index_first_child: u32,

    /// The character associated to this node.
    pub character: char,

    /// The word frequency. If None, the word does not exist in the dictionary.
    pub word_freq: Option<NonZeroU32>,
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
}
