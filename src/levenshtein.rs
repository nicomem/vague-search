use std::num::NonZeroU32;
use vague_search_core::{CompiledTrie, CompiledTrieNode, IndexNodeNonZero};

fn compare_keys(
    trie_node: &CompiledTrieNode,
    character: char,
    trie: &CompiledTrie,
) -> std::cmp::Ordering {
    match trie_node {
        CompiledTrieNode::PatriciaNode(node) => trie
            .get_chars(&node.char_range)
            .chars()
            .next()
            .unwrap()
            .cmp(&character),
        CompiledTrieNode::NaiveNode(node) => node.character.cmp(&character),
        CompiledTrieNode::RangeNode(node) => {
            let ranges = trie.get_range(&node.range);
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

pub fn distance_zero(
    trie: &CompiledTrie,
    word: &str,
    index: Option<IndexNodeNonZero>,
) -> Option<NonZeroU32> {
    let mut word_cpy = word;
    let mut children = match index {
        None => trie.get_root_siblings()?,
        Some(i) => trie.get_siblings(i),
    };

    loop {
        let first_char: char = word_cpy.chars().next().unwrap();

        let index_child = children
            .binary_search_by(|node| compare_keys(node, first_char, trie))
            .ok()?;

        let child = unsafe { children.get_unchecked(index_child) };

        children = match child {
            CompiledTrieNode::PatriciaNode(node) => {
                let chars = trie.get_chars(&node.char_range);
                let lenchar = chars.len();
                if lenchar > word_cpy.len() || !word_cpy.starts_with(chars) {
                    return None;
                }
                if word_cpy.len() == lenchar {
                    return node.word_freq;
                }
                // If no more childs then no more iterations
                if let Some(index) = node.index_first_child {
                    // Split word for next iteration
                    word_cpy = word_cpy.split_at(lenchar).1;
                    // Get the siblings for the next iteration
                    trie.get_siblings(index)
                } else {
                    return None;
                }
            }
            CompiledTrieNode::NaiveNode(node) => {
                if word_cpy.len() == node.character.len_utf8() {
                    return node.word_freq;
                }
                if let Some(index) = node.index_first_child {
                    word_cpy = word_cpy.split_at(node.character.len_utf8()).1;
                    trie.get_siblings(index)
                } else {
                    return None;
                }
            }
            CompiledTrieNode::RangeNode(node) => {
                let range = trie
                    .get_range(&node.range)
                    .get(first_char as usize - node.first_char as usize)?;
                if word_cpy.len() == first_char.len_utf8() {
                    return range.word_freq;
                }
                // Get next child index
                if let Some(index) = range.index_first_child {
                    word_cpy = word_cpy.split_at(first_char.len_utf8()).1;
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
    use std::ops::Range;
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

    fn create_range(range: Range<char>, step: usize) -> (Vec<char>, Vec<NodeDrainer>) {
        let chars: Vec<char> = ((range.start as u32)..(range.end as u32))
            .step_by(step)
            .flat_map(std::char::from_u32)
            .collect();

        let nodes = chars
            .iter()
            .map(|&c| c)
            .map(|c| create_simple(c, 0, vec![]))
            .collect();

        (chars, nodes)
    }

    #[test]
    fn mixed_search() {
        let root = create_simple(
            '-',
            0,
            vec![
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

        let search_cata = distance_zero(&compiled, "cata", None);
        assert!(search_cata.is_some());
        assert_eq!(search_cata.unwrap(), NonZeroU32::new(1).unwrap());

        let search_da = distance_zero(&compiled, "da", None);
        assert!(search_da.is_some());
        assert_eq!(search_da.unwrap(), NonZeroU32::new(9).unwrap());

        let search_dfade = distance_zero(&compiled, "fade", None);
        assert!(search_dfade.is_some());
        assert_eq!(search_dfade.unwrap(), NonZeroU32::new(10).unwrap());
    }
}
