use super::index::*;
use std::num::NonZeroU32;

use std::{fmt::Debug, ops::Range};

/// A [CompiledTrie](crate::CompiledTrie) node following a naive trie structure.
///
/// Node following the structure of a naive trie.
/// More efficient to hold one-character strings (e.g. a-f-i-z).
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct NaiveNode {
    /// The index of the first child in the node array.
    pub index_first_child: Option<IndexNodeNonZero>,

    /// The word frequency. If None, the word does not exist in the dictionary.
    pub word_freq: Option<NonZeroU32>,

    /// The character associated to this node.
    pub character: char,
}

/// A [CompiledTrie](crate::CompiledTrie) node following a Patricia trie structure.
///
/// Node following the structure of a PATRICIA trie.
/// More efficient to hold multiple-characters strings (e.g. bar-foo).
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PatriciaNode {
    /// The index of the first child in the node array.
    pub index_first_child: Option<IndexNodeNonZero>,

    /// The word frequency. If None, the word does not exist in the dictionary.
    pub word_freq: Option<NonZeroU32>,

    /// The start index of characters associated to this node in the characters array.
    /// The length of the stored string is stored inside its [CompiledTrieNode](crate::CompiledTrieNode).
    pub start_index: IndexChar,
}

/// A [CompiledTrie](crate::CompiledTrie) node representing a range of characters.
/// This node only represents the range of characters, to access its children,
/// check the [RangeSlice](crate::RangeSlice) of the [CompiledTrie](crate::CompiledTrie).
///
/// Node representing a range of characters where children are stored
/// in the range array.
/// More efficient for continuous range of 1-character nodes (e.g. a-b-c-d).
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RangeNode {
    /// The first character represented by this node.
    pub first_char: char,

    /// The start index of the range in the range array.
    pub start_index: IndexRange,

    /// The *exclusive* end index of the range in the range array.
    pub end_index: IndexRange,
}

/// An element of the range array, accessible via a [RangeNode](crate::RangeNode).
/// Since `index_first_child` cannot have the value 0, the struct can be contained
/// inside an Option without using more memory.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct RangeElement {
    /// The index of the first child in the node array.
    /// This index could not be equal to 0 because the 0th node is the trie root,
    /// which is a child to none.
    pub index_first_child: Option<IndexNodeNonZero>,

    /// The word frequency. If None, the word does not exist in the dictionary.
    pub word_freq: Option<NonZeroU32>,
}

/// A union of all possible node structures a node can have.
#[derive(Copy, Clone)]
union NodeUnion {
    naive: NaiveNode,
    patricia: PatriciaNode,
    range: RangeNode,
}

/// An enumeration of all possible node types.
pub enum NodeValue<'a> {
    Naive(&'a NaiveNode),
    Patricia(&'a PatriciaNode),
    Range(&'a RangeNode),
}

pub(super) enum NodeValueMut<'a> {
    Naive(&'a mut NaiveNode),
    Patricia(&'a mut PatriciaNode),
    Range(&'a mut RangeNode),
}

/// A node of a compiled trie.
/// Can be of different structure depending on the situation to optimize
/// memory consumption and execution speed.
#[derive(Clone)]
pub struct CompiledTrieNode {
    /// Store the number of siblings this node has but also store flags in its first bits.
    /// Bit structure (MSB):
    /// - 2 bits : [0-1]  : Type of node value in the union
    /// - 12 bits: [2-13] : **PatriciaNode** Length of the stored string
    /// - 18 bits: [14-31]: number of siblings at the **right** of this node
    /// Storing the number of siblings in only 18 bits is safe because there are
    /// 143,859 characters in the current Unicode version (13.0.0) which is much
    /// less than 2^18 = 262 144.
    /// In comparison, this last Unicode version added around 6000 new characters,
    /// and version 12.0.0 added only 554 new ones, so 18 bits should be
    /// future-proof for quite some time.
    nb_siblings_with_flags: u32,

    /// The main node information of the node.
    /// Corresponding flags of the `nb_siblings_with_flags` field must be checked
    /// to determine the union structure used.
    node_union: NodeUnion,
}

impl CompiledTrieNode {
    const MASK_NODE_TYPE: u32 = 0xC000_0000; // 1100..0000..0000
    const MASK_PAT_STR_LENGTH: u32 = 0x3FFC_0000; // 0011..1100..0000
    const MASK_NB_SIBLINGS: u32 = 0x0003_FFFF; // 0000..0011..0000

    const NODE_TYPE_NAIVE: u32 = 0; // 0b0000..
    const NODE_TYPE_PATRICIA: u32 = 0x4000_0000; // 0b0100..
    const NODE_TYPE_RANGE: u32 = 0x8000_0000; // 0b1000..

    pub fn new_naive(data: NaiveNode, nb_siblings: u32) -> Self {
        debug_assert!(
            nb_siblings < (1 << 18),
            "Too many siblings: {} >= {}",
            nb_siblings,
            1 << 18
        );
        Self {
            nb_siblings_with_flags: Self::NODE_TYPE_NAIVE
                | Self::value_to_mask(nb_siblings, Self::MASK_NB_SIBLINGS),
            node_union: NodeUnion { naive: data },
        }
    }

    pub fn new_patricia(data: PatriciaNode, nb_siblings: u32, str_len: u32) -> Self {
        debug_assert!(
            nb_siblings < (1 << 18),
            "Too many siblings: {} >= {}",
            nb_siblings,
            1 << 18
        );
        debug_assert!(
            str_len < (1 << 12),
            "String too long: {} >= {}",
            str_len,
            1 << 12
        );
        Self {
            nb_siblings_with_flags: Self::NODE_TYPE_PATRICIA
                | Self::value_to_mask(nb_siblings, Self::MASK_NB_SIBLINGS)
                | Self::value_to_mask(str_len, Self::MASK_PAT_STR_LENGTH),
            node_union: NodeUnion { patricia: data },
        }
    }

    pub fn new_range(data: RangeNode, nb_siblings: u32) -> Self {
        debug_assert!(
            nb_siblings < (1 << 18),
            "Too many siblings: {} >= {}",
            nb_siblings,
            1 << 18
        );
        Self {
            nb_siblings_with_flags: Self::NODE_TYPE_RANGE
                | Self::value_to_mask(nb_siblings, Self::MASK_NB_SIBLINGS),
            node_union: NodeUnion { range: data },
        }
    }

    /// Get the masked value contained in the `nb_siblings_with_flags` field.
    const fn get_masked_flags(&self, mask: u32) -> u32 {
        (self.nb_siblings_with_flags & mask) >> mask.trailing_zeros()
    }

    /// Transform the value to correspond to the given mask.
    const fn value_to_mask(value: u32, mask: u32) -> u32 {
        (value << mask.trailing_zeros()) & mask
    }

    /// Return the inner value of the node.
    pub fn node_value(&self) -> NodeValue {
        use std::hint::unreachable_unchecked;

        // SAFETY: The node type indicates the value structure of the union
        // SAFETY: It can only be one of the 3 different values below
        unsafe {
            match self.nb_siblings_with_flags & Self::MASK_NODE_TYPE {
                Self::NODE_TYPE_NAIVE => NodeValue::Naive(&self.node_union.naive),
                Self::NODE_TYPE_PATRICIA => NodeValue::Patricia(&self.node_union.patricia),
                Self::NODE_TYPE_RANGE => NodeValue::Range(&self.node_union.range),
                _ => unreachable_unchecked(),
            }
        }
    }

    /// Return the mutable inner value of the node.
    pub(super) fn node_value_mut(&mut self) -> NodeValueMut {
        use std::hint::unreachable_unchecked;

        // SAFETY: The node type indicates the value structure of the union
        // SAFETY: It can only be one of the 3 different values below
        unsafe {
            match self.nb_siblings_with_flags & Self::MASK_NODE_TYPE {
                Self::NODE_TYPE_NAIVE => NodeValueMut::Naive(&mut self.node_union.naive),
                Self::NODE_TYPE_PATRICIA => NodeValueMut::Patricia(&mut self.node_union.patricia),
                Self::NODE_TYPE_RANGE => NodeValueMut::Range(&mut self.node_union.range),
                _ => unreachable_unchecked(),
            }
        }
    }

    /// # Safety
    /// Undefined if not a patricia node.
    ///
    /// Return the length of the stored string.
    pub unsafe fn patricia_range(&self) -> Range<IndexChar> {
        debug_assert!(matches!(self.node_value(), NodeValue::Patricia(_)));
        let pat_str_len = self.get_masked_flags(Self::MASK_PAT_STR_LENGTH);

        let start_index: IndexChar = self.node_union.patricia.start_index;
        let end_index = IndexChar::new(*start_index + pat_str_len);
        start_index..end_index
    }

    /// Return the number of siblings of the node.
    pub const fn nb_siblings(&self) -> u32 {
        self.get_masked_flags(Self::MASK_NB_SIBLINGS)
    }
}

impl Debug for CompiledTrieNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (node, pat_range): (Box<dyn Debug>, Option<Range<IndexChar>>) = match self.node_value()
        {
            NodeValue::Naive(n) => (Box::new(n), None),
            // SAFETY: Safe because in a patricia node
            NodeValue::Patricia(n) => (Box::new(n), Some(unsafe { self.patricia_range() })),
            NodeValue::Range(n) => (Box::new(n), None),
        };

        f.debug_struct("CompiledTrieNode")
            .field("nb_siblings", &self.nb_siblings())
            .field("patricia_range", &pat_range)
            .field("node_union", &node)
            .finish()
    }
}

impl PartialEq for CompiledTrieNode {
    fn eq(&self, other: &Self) -> bool {
        if self.nb_siblings_with_flags != other.nb_siblings_with_flags {
            return false;
        }

        match (self.node_value(), other.node_value()) {
            (NodeValue::Naive(a), NodeValue::Naive(b)) => a == b,
            (NodeValue::Patricia(a), NodeValue::Patricia(b)) => a == b,
            (NodeValue::Range(a), NodeValue::Range(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for CompiledTrieNode {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_trie_nodes_correct_type() {
        let naive = CompiledTrieNode::new_naive(
            NaiveNode {
                index_first_child: None,
                word_freq: None,
                character: 'b',
            },
            3,
        );
        assert!(matches!(naive.node_value(), NodeValue::Naive(_)));

        let patricia = CompiledTrieNode::new_patricia(
            PatriciaNode {
                index_first_child: None,
                word_freq: None,
                start_index: IndexChar::new(0),
            },
            3,
            6,
        );
        assert!(matches!(patricia.node_value(), NodeValue::Patricia(_)));

        let range = CompiledTrieNode::new_range(
            RangeNode {
                start_index: IndexRange::new(0),
                end_index: IndexRange::new(5),
                first_char: 'b',
            },
            3,
        );
        assert!(matches!(range.node_value(), NodeValue::Range(_)));
    }

    #[test]
    fn test_patricia_nb_siblings_and_str_len() {
        let patricia = CompiledTrieNode::new_patricia(
            PatriciaNode {
                index_first_child: None,
                word_freq: None,
                start_index: IndexChar::new(0),
            },
            0,
            0,
        );
        assert_eq!(patricia.nb_siblings(), 0);
        assert_eq!(
            unsafe { patricia.patricia_range() },
            IndexChar::new(0)..IndexChar::new(0)
        );

        let patricia = CompiledTrieNode::new_patricia(
            PatriciaNode {
                index_first_child: NonZeroU32::new(995).map(IndexNodeNonZero::new),
                word_freq: NonZeroU32::new(875347),
                start_index: IndexChar::new(40),
            },
            3,
            3064,
        );
        assert_eq!(patricia.nb_siblings(), 3);
        assert_eq!(
            unsafe { patricia.patricia_range() },
            IndexChar::new(40)..IndexChar::new(3104)
        );

        let patricia = CompiledTrieNode::new_patricia(
            PatriciaNode {
                index_first_child: NonZeroU32::new(995).map(IndexNodeNonZero::new),
                word_freq: NonZeroU32::new(875347),
                start_index: IndexChar::new(0),
            },
            262143,
            4095,
        );
        assert_eq!(patricia.nb_siblings(), 262143);
        assert_eq!(
            unsafe { patricia.patricia_range() },
            IndexChar::new(0)..IndexChar::new(4095)
        );
    }
}
