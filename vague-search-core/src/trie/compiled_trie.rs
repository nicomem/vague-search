use crate::CompiledTrieNode;
use std::borrow::Cow;

/// Represent a valid index in the [CompiledTrie](CompiledTrie) node array.
#[derive(Debug, Copy, Clone)]
pub struct IndexNode {
    index: u32,
}

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
    nodes: Cow<'a, [CompiledTrieNode]>,
    chars: Cow<'a, [char]>,
}

impl CompiledTrie<'_> {
    /// Return a slice of the node array.
    pub(crate) fn nodes(&self) -> &[CompiledTrieNode] {
        &self.nodes
    }

    /// Return a slice of the character array.
    /// Does not include characters of [SimpleNode](CompiledTrieNode::SimpleNode).
    pub(crate) fn chars(&self) -> &[char] {
        &self.chars
    }

    /// Get the node corresponding to the index.
    pub fn get_node(&self, index: IndexNode) -> &CompiledTrieNode {
        // Safe because IndexNode inner value can only be set in this file
        unsafe { self.nodes.get_unchecked(index.index as usize) }
    }

    /// Get the index of the root node.
    pub const fn index_root(&self) -> IndexNode {
        IndexNode { index: 0 }
    }

    /// Get the index of the first children of the current index.
    pub fn index_child(&self, index: IndexNode) -> IndexNode {
        let index = match self.get_node(index) {
            CompiledTrieNode::NaiveNode(node) => node.index_first_child,
            CompiledTrieNode::PatriciaNode(node) => node.index_first_child,
        };
        IndexNode { index }
    }

    /// Try to get the index of a sibling of the current index.
    /// If the offset is out-of-bound, return None.
    pub fn index_sibling(&self, index: IndexNode, sibling_offset: u32) -> Option<IndexNode> {
        let nb_siblings = match self.get_node(index) {
            CompiledTrieNode::NaiveNode(node) => node.nb_siblings,
            CompiledTrieNode::PatriciaNode(node) => node.nb_siblings,
        };

        if sibling_offset >= nb_siblings {
            None
        } else {
            Some(IndexNode {
                index: index.index + sibling_offset,
            })
        }
    }

    /// Same as [index_sibling](CompiledTrie::index_sibling) but no out-of-bound check is done.
    pub unsafe fn index_sibling_unchecked(
        &self,
        index: IndexNode,
        sibling_offset: u32,
    ) -> IndexNode {
        IndexNode {
            index: index.index + sibling_offset,
        }
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
