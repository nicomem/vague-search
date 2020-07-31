use std::num::NonZeroU32;
use vague_search_core::{CompiledTrie, CompiledTrieNode, IndexNodeNonZero, NodeValue};

pub fn compare_keys(
    trie_node: &CompiledTrieNode,
    character: char,
    trie: &CompiledTrie,
) -> std::cmp::Ordering {
    match trie_node.node_value() {
        NodeValue::Naive(node) => node.character.cmp(&character),
        NodeValue::Patricia(_) => {
            // SAFETY: Safe because in a patricia node
            let pat_range = unsafe { &trie_node.patricia_range() };
            trie.get_chars(pat_range.start, pat_range.end)
                .chars()
                .next()
                .unwrap()
                .cmp(&character)
        }
        NodeValue::Range(node) => {
            let ranges = trie.get_range(node.start_index, node.end_index);
            if node.first_char > character {
                std::cmp::Ordering::Greater
            } else if node.first_char as usize + ranges.len() < character as usize {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        }
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

        let index_child = children
            .binary_search_by(|node| compare_keys(node, first_char, trie))
            .ok()?;

        let child = unsafe { children.get_unchecked(index_child) };

        children = match child.node_value() {
            NodeValue::Naive(node) => {
                if word.len() == node.character.len_utf8() {
                    return node.word_freq;
                }
                if let Some(index) = node.index_first_child {
                    word = word.split_at(node.character.len_utf8()).1;
                    trie.get_siblings(index)
                } else {
                    return None;
                }
            }
            NodeValue::Patricia(node) => {
                // SAFETY: Safe because in a patricia node
                let patricia_range = unsafe { child.patricia_range() };
                let chars = trie.get_chars(patricia_range.start, patricia_range.end);
                let lenchar = chars.len();
                if lenchar > word.len() || !word.starts_with(chars) {
                    return None;
                }
                if word.len() == lenchar {
                    return node.word_freq;
                }
                // If no more childs then no more iterations
                if let Some(index) = node.index_first_child {
                    // Split word for next iteration
                    word = word.split_at(lenchar).1;
                    // Get the siblings for the next iteration
                    trie.get_siblings(index)
                } else {
                    return None;
                }
            }
            NodeValue::Range(node) => {
                let range = trie
                    .get_range(node.start_index, node.end_index)
                    .get(first_char as usize - node.first_char as usize)?;
                if word.len() == first_char.len_utf8() {
                    return range.word_freq;
                }
                // Get next child index
                if let Some(index) = range.index_first_child {
                    word = word.split_at(first_char.len_utf8()).1;
                    trie.get_siblings(index)
                } else {
                    return None;
                }
            }
        }
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
