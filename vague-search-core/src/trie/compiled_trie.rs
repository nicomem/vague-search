use super::index::*;
use crate::CompiledTrieNode;
use std::{borrow::Cow, ops::Range};

pub type NodeSlice = [CompiledTrieNode];
pub type CharsSlice = str;
pub type RangeSlice = [RangeElement];

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
    pub(super) nodes: Cow<'a, NodeSlice>,
    pub(super) chars: Cow<'a, CharsSlice>,
    pub(super) ranges: Cow<'a, RangeSlice>,
}

impl CompiledTrie<'_> {
    /// Return a slice of the node array.
    pub(crate) fn nodes(&self) -> &NodeSlice {
        &self.nodes
    }

    /// Return a slice of the character array.
    pub(crate) fn chars(&self) -> &CharsSlice {
        &self.chars
    }

    /// Return a slice of the ranges array.
    pub(crate) fn ranges(&self) -> &RangeSlice {
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
    pub fn get_chars(&self, range: Range<IndexChar>) -> Option<&CharsSlice> {
        self.chars.get(range.start.into()..range.end.into())
    }

    /// Get a range of nodes corresponding to a [RangeNode](crate::RangeNode).
    pub fn get_range(&self, range: Range<IndexRange>) -> Option<&RangeSlice> {
        self.ranges.get(range.start.into()..range.end.into())
    }
}

impl<'a> From<(&'a NodeSlice, &'a CharsSlice, &'a RangeSlice)> for CompiledTrie<'a> {
    fn from((nodes, chars, ranges): (&'a NodeSlice, &'a CharsSlice, &'a RangeSlice)) -> Self {
        CompiledTrie {
            nodes: Cow::Borrowed(nodes),
            chars: Cow::Borrowed(chars),
            ranges: Cow::Borrowed(ranges),
        }
    }
}
