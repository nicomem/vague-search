use super::index::*;
use crate::CompiledTrieNode;
use std::{borrow::Cow, ops::Index};

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
///   nodes can be stored in an array, with each node holding the
#[derive(Debug, Clone)]
pub struct CompiledTrie<'a> {
    nodes: Cow<'a, NodeSlice>,
    chars: Cow<'a, CharSlice>,
    ranges: Cow<'a, RangeSlice>,
}

impl CompiledTrie<'_> {
    /// Return a slice of the node array.
    pub(crate) fn nodes(&self) -> &NodeSlice {
        &self.nodes
    }

    /// Return a slice of the character array.
    pub(crate) fn chars(&self) -> &CharSlice {
        &self.chars
    }

    /// Return a slice of the ranges array.
    pub(crate) fn ranges(&self) -> &RangeSlice {
        &self.ranges
    }

    /// Get the index of the root node.
    pub const fn index_root(&self) -> IndexNode {
        IndexNode::zero()
    }

    /// Try to get the index of the first child of a sibling of the current index.
    /// If the offset is out-of-bound, return None.
    pub fn index_child_of_sibling(
        &self,
        index: IndexNode,
        sibling_offset: u32,
    ) -> Option<IndexNode> {
        if sibling_offset >= self[index].nb_siblings() {
            None
        } else {
            Some(index.offset_unchecked(sibling_offset))
        }
    }

    /// Same as [index_child_of_sibling](CompiledTrie::index_child_of_sibling)
    /// but **no out-of-bound check is done**.
    pub fn index_child_of_sibling_unchecked(
        &self,
        index: IndexNode,
        sibling_offset: u32,
    ) -> IndexNode {
        index.offset_unchecked(sibling_offset)
    }
}

impl<'a> From<(&'a NodeSlice, &'a CharSlice, &'a RangeSlice)> for CompiledTrie<'a> {
    fn from((nodes, chars, ranges): (&'a NodeSlice, &'a CharSlice, &'a RangeSlice)) -> Self {
        CompiledTrie {
            nodes: Cow::Borrowed(nodes),
            chars: Cow::Borrowed(chars),
            ranges: Cow::Borrowed(ranges),
        }
    }
}

// TODO: impl From<Trie>

// Macro to implement trie indexing for subslices index wrappers
macro_rules! index_wrapper {
    ($index:ident, $field: ident, $elem:ty) => {
        impl Index<$index> for CompiledTrie<'_> {
            type Output = $elem;

            fn index(&self, index: $index) -> &Self::Output {
                &self.$field[index]
            }
        }
    };
}

index_wrapper!(IndexNode, nodes, CompiledTrieNode);
index_wrapper!(IndexChar, chars, char);
index_wrapper!(IndexRange, ranges, RangeElement);

// TODO?: Add method to get char slice from range of IndexChar
