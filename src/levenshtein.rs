use vague_search_core::{CompiledTrie, CompiledTrieNode};
use std::num::NonZeroU32;

fn compare_keys(
    trie_node: &CompiledTrieNode,
    character: char,
    trie: &CompiledTrie,
) -> std::cmp::Ordering {
    match trie_node {
        CompiledTrieNode::PatriciaNode(a) => trie
            .get_chars(&a.char_range)
            .chars()
            .next()
            .unwrap()
            .cmp(&character),
        CompiledTrieNode::NaiveNode(b) => b.character.cmp(&character),
        CompiledTrieNode::RangeNode(c) => {
            // FIXME
            let ranges = trie.get_range(&c.range);
            if c.first_char > character {
                std::cmp::Ordering::Less
            } else if c.first_char as usize + ranges.len() < character as usize {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        }
    }
}

pub fn distance_zero(trie: &CompiledTrie, word: String) -> Option<NonZeroU32> {
    let mut word_cpy = word.clone();
    let mut children = trie.get_root_siblings()?;
    dbg!(trie);
    loop {
        let first_char: char = word_cpy.chars().next().unwrap();
        dbg!(first_char);
        dbg!(children);

        let index_child =
            children.binary_search_by(|node| compare_keys(node, first_char, trie));

        let index = match index_child {
            Ok(u) => {u}
            Err(_) => return None
        };

        let child = children.get(index)?;

        children = match child {
            CompiledTrieNode::PatriciaNode(node) => {
                let chars = trie.get_chars(&node.char_range);
                let lenchar = chars.len();
                if lenchar > word_cpy.len() || !word_cpy.starts_with(chars) {break; }
                if word_cpy.len() == lenchar { return node.word_freq }
                // If no more childs then no more iterations
                if node.index_first_child.is_none() { break; }
                // Split word for next iteration
                word_cpy = word_cpy.split_off(lenchar);
                // Get the siblings for the next iteration
                trie.get_siblings(node.index_first_child.unwrap())
            }
            CompiledTrieNode::NaiveNode(node) => {
                if word_cpy.len() == 1 { return node.word_freq;}
                else if node.index_first_child.is_none() { break; }
                word_cpy = word_cpy.split_off(0);
                trie.get_siblings(node.index_first_child.unwrap())
            }
            CompiledTrieNode::RangeNode(node) => {
                let range = trie.get_range(&node.range).get(first_char as usize - node.first_char as usize)?;
                if word_cpy.len() == 1 { return range.word_freq;}
                else if range.index_first_child.is_none() { break; }

                word_cpy = word_cpy.split_off(0);
                trie.get_siblings(range.index_first_child.unwrap())

            }
        }
        
    }
    None
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
    fn simple_search() {
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

        let search_cata = distance_zero(&compiled, String::from("cata"));
        assert!(search_cata.is_some());
        assert_eq!(search_cata.unwrap(), NonZeroU32::new(1).unwrap());

        let search_da= distance_zero(&compiled, String::from("da"));
        assert!(search_da.is_some());
        assert_eq!(search_da.unwrap(), NonZeroU32::new(9).unwrap());

        let search_dfade= distance_zero(&compiled, String::from("dfade"));
        assert!(search_dfade.is_some());
        assert_eq!(search_dfade.unwrap(), NonZeroU32::new(10).unwrap());

    }
}
