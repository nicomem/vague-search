use super::index::*;
use crate::{CompiledTrieNode, RangeElement};
use std::{borrow::Cow, ops::Range};

/// Represent the node array of the [CompiledTrie](crate::CompiledTrie)
pub type NodeSlice = [CompiledTrieNode];

/// Represent the character array of the [CompiledTrie](crate::CompiledTrie)
pub type CharsSlice = str;

/// Represent the range array of the [CompiledTrie](crate::CompiledTrie)
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
    /// Only return None when the nodes array is empty.
    pub fn root(&self) -> Option<&CompiledTrieNode> {
        self.nodes.get(0)
    }

    /// Get a node and its siblings from the trie.
    pub fn get_siblings(&self, index: IndexNodeNonZero) -> &[CompiledTrieNode] {
        let start_index = usize::from(index);
        debug_assert!(start_index < self.nodes.len());

        // SAFETY: both IndexNodeNonZero are valid because they cannot be created by the user.
        unsafe {
            let first_node = self.nodes.get_unchecked(start_index);
            let end_index = start_index + first_node.nb_siblings() as usize;

            debug_assert!(end_index < self.nodes.len());
            self.nodes.get_unchecked(start_index..end_index)
        }
    }

    /// Get a range of characters of a [PatriciaNode](crate::PatriciaNode).
    pub fn get_chars(&self, range: Range<IndexChar>) -> &CharsSlice {
        // SAFETY: Both IndexChar are valid because they cannot be created by the user.
        unsafe {
            self.chars
                .get_unchecked(usize::from(range.start)..usize::from(range.end))
        }
    }

    /// Get a range of nodes corresponding to a [RangeNode](crate::RangeNode).
    pub fn get_range(&self, range: Range<IndexRange>) -> &RangeSlice {
        // SAFETY: Both IndexRange are valid because they cannot be created by the user.
        unsafe {
            self.ranges
                .get_unchecked(usize::from(range.start)..usize::from(range.end))
        }
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
