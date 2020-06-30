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
    pub(super) ranges: Cow<'a, [RangeElement]>,
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
    pub(crate) fn ranges(&self) -> &[RangeElement] {
        &self.ranges
    }

    /// Get a node and its *right* siblings from the trie.
    pub fn get_siblings(&self, index: IndexNode) -> Option<&[CompiledTrieNode]> {
        let nb_siblings = self
            .nodes
            .get(*index as usize)
            .map(CompiledTrieNode::nb_siblings);

        let range = nb_siblings.map(|len| *index as usize..(*index + len) as usize);

        range.map(|r| self.nodes.get(r)).flatten()
    }

    /// Get a range of characters of a [PatriciaNode](crate::PatriciaNode).
    pub fn get_chars(&self, range: Range<IndexChar>) -> Option<&str> {
        self.chars.get(*range.start as usize..*range.end as usize)
    }

    /// Get a range of nodes corresponding to a [RangeNode](crate::RangeNode).
    pub fn get_range(&self, range: Range<IndexRange>) -> Option<&[RangeElement]> {
        self.ranges.get(*range.start as usize..*range.end as usize)
    }

    /// Try to get the index of the first child of a sibling of the current index.
    /// If the offset is out-of-bound, return None.
    pub fn index_child_of_sibling(
        &self,
        current_index: IndexNode,
        sibling_offset: u32,
    ) -> Option<IndexNode> {
        self.nodes
            .get(*current_index as usize)
            .filter(|node| sibling_offset < node.nb_siblings() as u32)
            .map(|_| unsafe {
                self.index_child_of_sibling_unchecked(current_index, sibling_offset)
            })
    }

    /// Same as [index_child_of_sibling](crate::CompiledTrie::index_child_of_sibling)
    /// but no out-of-bound checks are done.
    pub unsafe fn index_child_of_sibling_unchecked(
        &self,
        current_index: IndexNode,
        sibling_offset: u32,
    ) -> IndexNode {
        IndexNode::new(*current_index + sibling_offset)
    }
}

impl<'a> From<(&'a [CompiledTrieNode], &'a str, &'a [RangeElement])> for CompiledTrie<'a> {
    fn from((nodes, chars, ranges): (&'a [CompiledTrieNode], &'a str, &'a [RangeElement])) -> Self {
        CompiledTrie {
            nodes: Cow::Borrowed(nodes),
            chars: Cow::Borrowed(chars),
            ranges: Cow::Borrowed(ranges),
        }
    }
}
