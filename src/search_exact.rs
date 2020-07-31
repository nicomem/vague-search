use std::{cmp::Ordering, num::NonZeroU32};
use vague_search_core::{CompiledTrie, CompiledTrieNode, IndexNodeNonZero, NodeValue};

/// Compare the node characters with the character.
/// If the character is in the node's range, return Equal.
/// If the character is before the node's range, return Greater.
pub fn compare_keys(
    trie_node: &CompiledTrieNode,
    node_value: &NodeValue,
    character: char,
    trie: &CompiledTrie,
) -> Ordering {
    match node_value {
        NodeValue::Naive(node) => node.character.cmp(&character),
        NodeValue::Patricia(_) => {
            // SAFETY: Safe because in a patricia node
            let pat_range = unsafe { &trie_node.patricia_range() };
            let pat_first_char = unsafe { trie.get_char_unchecked(pat_range.start) };
            pat_first_char.cmp(&character)
        }
        NodeValue::Range(node) => {
            let range_len = u32::from(node.end_index) - u32::from(node.start_index);
            if node.first_char > character {
                Ordering::Greater
            } else if node.first_char as u32 + range_len <= character as u32 {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        }
    }
}

fn search_child<'a>(
    children: &'a [CompiledTrieNode],
    first_char: char,
    trie: &CompiledTrie,
) -> Option<(&'a CompiledTrieNode, NodeValue<'a>)> {
    // Custom binsearch, derived from the std implementation
    // See https://doc.rust-lang.org/std/primitive.slice.html#method.binary_search_by
    let mut size = children.len();
    let mut base = 0usize;
    while size > 1 {
        let half = size / 2;
        let mid = base + half;

        let trie_node = unsafe { children.get_unchecked(mid) };
        let node_value = trie_node.node_value();

        // mid is always in [0, size), that means mid is >= 0 and < size.
        // mid >= 0: by definition
        // mid < size: mid = size / 2 + size / 4 + size / 8 ...
        match compare_keys(trie_node, &node_value, first_char, trie) {
            Ordering::Less => base = mid,
            Ordering::Equal => return Some((trie_node, node_value)),
            Ordering::Greater => {}
        }
        size -= half;
    }

    let trie_node = unsafe { children.get_unchecked(base) };
    let node_value = trie_node.node_value();

    // base is always in [0, size) because base <= mid.
    let cmp = compare_keys(trie_node, &node_value, first_char, trie);
    if cmp == Ordering::Equal {
        Some((trie_node, node_value))
    } else {
        None
    }
}

pub fn search_exact(
    trie: &CompiledTrie,
    word: &str,
    index: Option<IndexNodeNonZero>,
) -> Option<NonZeroU32> {
    let children = match index {
        None => trie.get_root_siblings()?,
        Some(i) => trie.get_siblings(i),
    };

    search_exact_children(trie, word, children)
}

pub fn search_exact_children<'a>(
    trie: &'a CompiledTrie,
    mut word: &str,
    mut children: &'a [CompiledTrieNode],
) -> Option<NonZeroU32> {
    loop {
        debug_assert_ne!(word.len(), 0);
        let first_char: char = word.chars().next()?;

        let (child, child_value) = search_child(children, first_char, trie)?;
        let (index_first_child, word_freq, substr_len) = match child_value {
            NodeValue::Naive(node) => (
                node.index_first_child,
                node.word_freq,
                node.character.len_utf8(),
            ),
            NodeValue::Patricia(node) => {
                // SAFETY: Safe because in a patricia node
                let patricia_range = unsafe { child.patricia_range() };
                let chars = trie.get_chars(patricia_range.start, patricia_range.end);

                if !word.starts_with(chars) {
                    return None;
                }
                (node.index_first_child, node.word_freq, chars.len())
            }
            NodeValue::Range(node) => {
                // SAFFETY: node.first_char is in the range (checked inside search_child)
                let range = unsafe {
                    trie.get_range_element_unchecked(
                        node.start_index,
                        first_char as usize - node.first_char as usize,
                    )
                };

                (
                    range.index_first_child,
                    range.word_freq,
                    first_char.len_utf8(),
                )
            }
        };

        word = &word[substr_len..];
        if word.is_empty() {
            return word_freq;
        }
        children = trie.get_siblings(index_first_child?);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use vague_search_core::TrieNodeDrainer;

    #[derive(Debug, Default, Clone, Eq, PartialEq)]
    struct NodeDrainer {
        pub characters: String,
        pub frequency: Option<NonZeroU32>,
        pub children: Vec<Self>,
    }

    impl TrieNodeDrainer for NodeDrainer {
        fn drain_characters(&mut self) -> String {
            std::mem::replace(&mut self.characters, String::new())
        }

        fn frequency(&self) -> Option<NonZeroU32> {
            self.frequency
        }

        fn drain_children(&mut self) -> Vec<Self> {
            std::mem::replace(&mut self.children, Vec::new())
        }
    }

    fn create_simple(character: char, freq: u32, children: Vec<NodeDrainer>) -> NodeDrainer {
        NodeDrainer {
            characters: character.to_string(),
            frequency: NonZeroU32::new(freq),
            children,
        }
    }

    fn create_patricia(s: &str, freq: u32, children: Vec<NodeDrainer>) -> NodeDrainer {
        NodeDrainer {
            characters: s.to_string(),
            frequency: NonZeroU32::new(freq),
            children,
        }
    }

    #[test]
    fn mixed_search() {
        let root = create_simple(
            '-',
            0,
            vec![
                create_simple('a', 0, vec![create_patricia("la", 20, vec![])]),
                create_simple('b', 1, vec![]),
                create_patricia(
                    "cata",
                    1,
                    vec![create_simple('d', 2, vec![]), create_simple('f', 1, vec![])],
                ),
                create_simple(
                    'd',
                    0,
                    vec![
                        create_simple('a', 9, vec![]),
                        create_simple('r', 6, vec![]),
                        create_simple('t', 1, vec![]),
                        create_simple('w', 7, vec![]),
                    ],
                ),
                create_simple('f', 5, vec![create_patricia("ade", 10, vec![])]),
            ],
        );
        let compiled = CompiledTrie::from(root);

        let search_cata = search_exact(&compiled, "cata", None);
        assert!(search_cata.is_some());
        assert_eq!(search_cata.unwrap(), NonZeroU32::new(1).unwrap());

        let search_da = search_exact(&compiled, "da", None);
        assert!(search_da.is_some());
        assert_eq!(search_da.unwrap(), NonZeroU32::new(9).unwrap());

        let search_dfade = search_exact(&compiled, "fade", None);
        assert!(search_dfade.is_some());
        assert_eq!(search_dfade.unwrap(), NonZeroU32::new(10).unwrap());

        let search_ala = search_exact(&compiled, "ala", None);
        assert!(search_ala.is_some());
        assert_eq!(search_ala.unwrap(), NonZeroU32::new(20).unwrap());
    }
}
