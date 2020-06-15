use crate::utils::AsBytes;
use std::borrow::Cow;

/// A node of a compiled trie.
/// A node is defined as an enumeration for speed and memory optimization reasons:
/// - PatriciaNode: More efficient to hold multiple-characters strings
/// - SimpleNode: More efficient to hold one-character strings
#[derive(Debug, Clone)]
pub enum CompiledTrieNode {
    /// Node following the structure of a PATRICIA trie
    PatriciaNode(/* TODO */),

    /// Node following the structure of a naive trie
    SimpleNode(/* TODO */),
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
