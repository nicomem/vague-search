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

    loop {
        let first_char = word_cpy.chars().next().unwrap();
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
