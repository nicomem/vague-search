use crate::{
    trie::trie_node_interface::TrieNodeInterface, utils::find_subslice, CompiledTrie,
    CompiledTrieNode, IndexChar, IndexNode, NaiveNode, PatriciaNode, RangeElement,
};
use std::{borrow::Cow, num::NonZeroUsize, ops::Range};

/// Add the characters to the vector and return its range of index.
/// If the characters are already present in the vector, it may not insert them
/// and instead return the already present characters range of index.
fn add_chars(vec: &mut Vec<char>, chars: &[char]) -> Range<IndexChar> {
    let dup_window = NonZeroUsize::new(2048).unwrap();

    let range = find_subslice(vec, chars, Some(dup_window)).unwrap_or_else(|| {
        vec.extend_from_slice(chars);
        (vec.len() - chars.len())..vec.len()
    });

    let start = IndexChar::new(range.start as u32);
    let end = IndexChar::new(range.end as u32);

    start..end
}

/// Append the information of the given node and its children
/// to the three [CompiledTrie](crate::CompiledTrie) vectors.
fn fill_from_trie<N: TrieNodeInterface>(
    node: &N,
    nodes: &mut Vec<CompiledTrieNode>,
    chars: &mut Vec<char>,
    ranges: &mut Vec<RangeElement>,
) {
    let children = node.children();
    for (i, child) in children.iter().enumerate() {
        let node_chars = child.characters();
        let nb_siblings = (children.len() - i - 1) as u32;
        let word_freq = child.frequency();

        // Dummy value since only known after recursion
        let index_first_child = IndexNode::new(0);

        let node = if chars.len() == 1 {
            CompiledTrieNode::NaiveNode(NaiveNode {
                nb_siblings,
                index_first_child,
                word_freq,
                character: chars[0],
            })
        } else {
            let char_range = add_chars(chars, node_chars);
            CompiledTrieNode::PatriciaNode(PatriciaNode {
                nb_siblings,
                index_first_child,
                word_freq,
                char_range,
            })
        };

        nodes.push(node);
    }
}

impl<N: TrieNodeInterface> From<&N> for CompiledTrie<'_> {
    fn from(root: &N) -> Self {
        const CHARS_INIT_CAP: usize = 256;
        const RANGES_INIT_CAP: usize = 64;

        let mut nodes = Vec::with_capacity(root.hint_nb_nodes());
        let mut chars = Vec::with_capacity(CHARS_INIT_CAP);
        let mut ranges = Vec::with_capacity(RANGES_INIT_CAP);

        fill_from_trie(root, &mut nodes, &mut chars, &mut ranges);

        Self {
            nodes: Cow::Owned(nodes),
            chars: Cow::Owned(chars),
            ranges: Cow::Owned(ranges),
        }
    }
}
