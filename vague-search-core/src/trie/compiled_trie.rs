use super::index::*;
use crate::{CompiledTrieNode, RangeElement};
use std::borrow::Cow;

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

    /// Return the root node and its siblings.
    /// Only return None when the nodes array is empty.
    pub fn get_root_siblings(&self) -> Option<&[CompiledTrieNode]> {
        if self.nodes.is_empty() {
            None
        } else {
            // SAFETY: Not empty, condition checked just before
            unsafe { Some(self.get_siblings_unchecked(0)) }
        }
    }

    /// Get a node and its siblings from the trie. Unsafe version
    unsafe fn get_siblings_unchecked(&self, index: usize) -> &[CompiledTrieNode] {
        debug_assert!(index < self.nodes.len());

        let first_node = self.nodes.get_unchecked(index);
        // Add +1 to count current node in the range
        let end_index = index + first_node.nb_siblings() as usize + 1;

        debug_assert!(end_index <= self.nodes.len());
        self.nodes.get_unchecked(index..end_index)
    }

    /// Get a node and its siblings from the trie.
    pub fn get_siblings(&self, index: IndexNodeNonZero) -> &[CompiledTrieNode] {
        // SAFETY: IndexNodeNonZero is valid because it cannot be created by the user.
        unsafe { self.get_siblings_unchecked(usize::from(index)) }
    }

    /// Get a range of characters of a [PatriciaNode](crate::PatriciaNode).
    pub fn get_chars(&self, start: IndexChar, end: IndexChar) -> &CharsSlice {
        if start >= end {
            Default::default()
        } else {
            // SAFETY: IndexChar is valid because it cannot be created by the user.
            // SAFETY: start < end so the range is not out-of-bound.
            unsafe {
                self.chars
                    .get_unchecked(usize::from(start)..usize::from(end))
            }
        }
    }

    /// # Safety
    /// The index must be valid and the beginning of a substring.
    ///
    /// Get a single char beginning at the given index.
    pub unsafe fn get_char_unchecked(&self, index: IndexChar) -> char {
        self.chars
            .get_unchecked(usize::from(index)..)
            .chars()
            .next()
            .unwrap_or_else(|| std::hint::unreachable_unchecked())
    }

    /// Get a range of nodes corresponding to a [RangeNode](crate::RangeNode).
    pub fn get_range(&self, start: IndexRange, end: IndexRange) -> &RangeSlice {
        if start >= end {
            Default::default()
        } else {
            // SAFETY: IndexRange is valid because it cannot be created by the user.
            // SAFETY: start < end so the range is not out-of-bound.
            unsafe {
                self.ranges
                    .get_unchecked(usize::from(start)..usize::from(end))
            }
        }
    }

    /// # Safety
    /// The start index must be valid and the beginning of a range.
    /// The offset must be strictly less than the range length.
    ///
    /// Get a single element of a range beginning at the given [IndexRange](IndexRange).
    /// Does not any bound check.
    pub unsafe fn get_range_element_unchecked(
        &self,
        start: IndexRange,
        offset: usize,
    ) -> &RangeElement {
        self.ranges.get_unchecked(usize::from(start) + offset)
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
