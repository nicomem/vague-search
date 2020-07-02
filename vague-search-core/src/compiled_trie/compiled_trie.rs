use super::index::*;
use crate::CompiledTrieNode;
use std::{borrow::Cow, ops::Range};

/// A trie data structure which has been optimized for size and speed.
/// These optimizations come at the cost of not being able to modify the trie.
///
/// This structure implements a [Patricia Trie](https://en.wikipedia.org/wiki/Radix_tree#PATRICIA),
/// and stored as a [left-child right-sibling binary tree (LCRSBT)](https://en.wikipedia.org/wiki/Left-child_right-sibling_binary_tree).
/// This implementation choice has many advantages:
/// - **Size**: A Patricia trie compacts multiple trie nodes into one holding
///   a string instead of a character, this reduces the number of nodes and thus
///   the memory consumption of the data structure.
/// - **Not nested**: Since the LCRSBT representation is a binary tree,
///   nodes can be stored in a contiguous array.
#[derive(Debug, Clone)]
pub struct CompiledTrie<'a> {
    pub(super) nodes: Cow<'a, [CompiledTrieNode]>,
    pub(super) chars: Cow<'a, str>,
    pub(super) ranges: Cow<'a, [Option<RangeElement>]>,
}

impl CompiledTrie<'_> {
    /// Return a slice of the node array.
    pub(crate) fn nodes(&self) -> &[CompiledTrieNode] {
        &self.nodes
    }

    /// Return a slice of the character array.
    pub(crate) fn chars(&self) -> &str {
        &self.chars
    }

    /// Return a slice of the ranges array.
    pub(crate) fn ranges(&self) -> &[Option<RangeElement>] {
        &self.ranges
    }

    /// Return the root node.
    pub fn root(&self) -> Option<&CompiledTrieNode> {
        self.nodes.get(0)
    }

    /// Get a node from the trie.
    pub fn get_node(&self, index: IndexNodeNonZero) -> Option<&CompiledTrieNode> {
        self.nodes.get(usize::from(index))
    }

    /// Get a range of characters of a [PatriciaNode](crate::PatriciaNode).
    pub fn get_chars(&self, range: Range<IndexChar>) -> Option<&str> {
        self.chars.get(range.start.into()..range.end.into())
    }

    /// Get a range of nodes corresponding to a [RangeNode](crate::RangeNode).
    pub fn get_range(&self, range: Range<IndexRange>) -> Option<&[Option<RangeElement>]> {
        self.ranges.get(range.start.into()..range.end.into())
    }
}

impl<'a> From<(&'a [CompiledTrieNode], &'a str, &'a [Option<RangeElement>])> for CompiledTrie<'a> {
    fn from(
        (nodes, chars, ranges): (&'a [CompiledTrieNode], &'a str, &'a [Option<RangeElement>]),
    ) -> Self {
        CompiledTrie {
            nodes: Cow::Borrowed(nodes),
            chars: Cow::Borrowed(chars),
            ranges: Cow::Borrowed(ranges),
        }
    }
}
