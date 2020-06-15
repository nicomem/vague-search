use crate::utils::AsBytes;
use std::{borrow::Cow, num::NonZeroU32};

/// **TODO: Check size + reduce fields size**
///
/// A [CompiledTrie](CompiledTrie) node following a Patricia trie structure.
#[derive(Debug, Clone)]
pub struct PatriciaNode {
    /// The number of siblings of the node.
    /// The next sibling is located at the next index in the node array.
    nb_siblings: usize,

    /// The index of the first child in the node array.
    index_first_child: usize,

    /// The index of the first character in the characters array.
    index_first_char: usize,

    /// The number of characters associated to this node.
    nb_chars: usize,

    /// The word frequency. If None, the word does not exist in the dictionary.
    word_freq: Option<NonZeroU32>,
}

/// **TODO: Check size + reduce fields size**
///
/// A [CompiledTrie](CompiledTrie) node following a naive trie structure.
#[derive(Debug, Clone)]
pub struct NaiveNode {
    /// The number of siblings of the node.
    /// The next sibling is located at the next index in the node array.
    nb_siblings: usize,

    /// The index of the first child in the node array.
    index_first_child: usize,

    /// The index of the first character in the characters array.
    index_first_char: usize,

    /// The character associated to this node.
    character: char,
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

/// A trie data structure which has been optimized for size and speed.
/// These optimizations come at the cost of not being able to modify the trie.
///
/// This structure implements a [Patricia Trie](https://en.wikipedia.org/wiki/Radix_tree#PATRICIA),
/// and stored in a [left-child right-sibling binary tree (LCRSBT)](https://en.wikipedia.org/wiki/Left-child_right-sibling_binary_tree).
/// This implementation choice has many advantages:
/// - **Size**: A Patricia trie compacts multiple trie nodes into one holding
///   a string instead of a character, this reduces the number of nodes and thus
///   the memory consumption of the data structure.
/// - **Not nested**: Since the LCRSBT representation is a binary tree,
///   nodes can be stored in an array, with each node holding the
#[derive(Debug, Clone)]
pub struct CompiledTrie<'a> {
    nodes: Cow<'a, [CompiledTrieNode]>,
    chars: Cow<'a, [char]>,
}

impl CompiledTrie<'_> {
    pub fn nodes_len(&self) -> usize {
        self.nodes.len()
    }

    pub fn chars_len(&self) -> usize {
        self.chars.len()
    }

    pub fn nodes_bytes(&self) -> &[u8] {
        unsafe { self.nodes.as_bytes() }
    }

    pub fn chars_bytes(&self) -> &[u8] {
        unsafe { self.chars.as_bytes() }
    }
}

impl<'a> From<(&'a [CompiledTrieNode], &'a [char])> for CompiledTrie<'a> {
    fn from((nodes, chars): (&'a [CompiledTrieNode], &'a [char])) -> Self {
        CompiledTrie {
            nodes: Cow::Borrowed(nodes),
            chars: Cow::Borrowed(chars),
        }
    }
}

// TODO: impl From<Trie>
