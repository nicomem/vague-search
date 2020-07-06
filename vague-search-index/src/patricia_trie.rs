use crate::error::*;
use crate::utils::read_lines;
use snafu::*;
use std::{cmp::Ordering, num::NonZeroU32, path::Path};
use vague_search_core::TrieNodeDrainer;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PatriciaNode {
    letters: String,
    children: Vec<PatriciaNode>,
    freq: Option<NonZeroU32>,
}

pub fn index_difference(first: &str, second: &str) -> Option<usize> {
    first.chars().zip(second.chars()).position(|(a, b)| a != b)
}

impl PatriciaNode {
    pub(crate) fn create_empty() -> Self {
        Self {
            letters: String::new(),
            children: Vec::new(),
            freq: None,
        }
    }

    pub(crate) fn create_from_file(filepath: impl AsRef<Path>) -> Result<Self> {
        let path = filepath.as_ref();
        let mut root = Self::create_empty();
        let lines = read_lines(path).context(FileOpen { path })?;
        for (number, line) in lines.enumerate() {
            let wordfreq = line.context(FileRead { path })?;
            let mut iter = wordfreq.split_whitespace();
            // Parse word
            let word = iter.next().context(ContentRead {
                path,
                line: &wordfreq,
                number,
            })?;

            // Parse frequency
            let freqstr = iter.next().context(ContentRead {
                path,
                line: &wordfreq,
                number,
            })?;
            let freq = freqstr
                .parse::<NonZeroU32>()
                .context(Parsing { path, number })?;
            root.insert(word, freq)
        }
        Ok(root)
    }

    /// Divides a node by two in indicated index and creates the childs accordingly
    fn divide_node(&mut self, word: &str, ind: usize, frequency: NonZeroU32) {
        // Divide the current node into the current and a new one
        let second_part = self.letters.split_off(ind);
        let second_part_node = PatriciaNode {
            letters: second_part,
            children: std::mem::replace(&mut self.children, Vec::new()),
            freq: self.freq.take(),
        };

        // Remove the same prefix from the word to insert
        let (_, second_word) = word.split_at(ind);

        if second_word.is_empty() {
            // If the word to insert consisted only of the prefix,
            // mark the current node as an end node and set the second part node
            // as the only child
            self.freq = Some(frequency);
            self.children = vec![second_part_node];
        } else {
            // Create the word node and set the children as both nodes,
            // sorted by their letters
            let new_word_node = PatriciaNode {
                letters: second_word.to_string(),
                children: Vec::new(),
                freq: Some(frequency),
            };

            let sec_first_char = second_part_node.letters.chars().next().unwrap();
            let new_first_char = new_word_node.letters.chars().next().unwrap();
            match new_first_char.cmp(&sec_first_char) {
                Ordering::Less => self.children = vec![new_word_node, second_part_node],
                Ordering::Equal => unreachable!(),
                Ordering::Greater => self.children = vec![second_part_node, new_word_node],
            }
        }
    }

    fn create_and_insert_at(&mut self, index: usize, word: &str, frequency: NonZeroU32) {
        let child = PatriciaNode {
            letters: word.to_string(),
            children: Vec::new(),
            freq: Some(frequency),
        };
        self.children.insert(index, child);
    }

    fn divide(&mut self, word: &str, frequency: NonZeroU32) -> bool {
        let index_diff = index_difference(&self.letters, &word);

        match (index_diff, word.len().cmp(&self.letters.len())) {
            (Some(ind), _) => {
                self.divide_node(word, ind, frequency);
                true
            }
            (None, std::cmp::Ordering::Less) => {
                self.divide_node(word, word.len(), frequency);
                true
            }
            (None, std::cmp::Ordering::Equal) => {
                self.freq = Some(frequency);
                true
            }
            (None, _) => false,
        }
    }

    /// Insert a word and its frequency in the patricia trie
    pub(crate) fn insert(&mut self, word: impl Into<String>, frequency: NonZeroU32) {
        // Clone to avoid destroying given data
        let mut word_cpy = word.into();

        // No need of doing anything if the word is empty
        if word_cpy.is_empty() {
            return;
        }

        // Mutable pointer to switch between the parents and children
        let mut parent: &mut PatriciaNode = self;

        loop {
            let res = parent.children.binary_search_by(|child| {
                child.letters.chars().next().cmp(&word_cpy.chars().next())
            });

            let index_child = match res {
                Ok(r) => {
                    let child = &mut parent.children[r];
                    let insrt = child.divide(&word_cpy, frequency);
                    if !insrt {
                        word_cpy = word_cpy.split_off(child.letters.len());
                        Some(r)
                    } else {
                        None
                    }
                }
                Err(r) => {
                    parent.create_and_insert_at(r, &word_cpy, frequency);
                    None
                }
            };

            match index_child {
                Some(i) => parent = &mut parent.children[i],
                None => break,
            }
        }
    }

    #[cfg(test)]
    fn delete_node(&mut self, word: &str, index: usize) -> bool {
        let child = self.children.get_mut(index).unwrap();

        if child.letters.len() < word.len() {
            return !word.starts_with(child.letters.as_str()); // false to continue looping
        } else if child.letters.len() > word.len() || child.freq == None {
            return true;
        }

        // Both words are not equal, consider as deleted node
        if child.letters != word {
            return true;
        }

        // If more than one children only removing the node as a word is sufficient
        if child.children.len() > 1 {
            child.freq = None;
        }
        // Otherwise remove the node entirely
        else if child.children.is_empty() {
            self.children.remove(index);
        }
        // Or combine the child and its only child
        else {
            let mut leftover_child = child.children.pop().unwrap();
            child.letters.push_str(leftover_child.letters.as_str());
            child.freq = leftover_child.freq;
            std::mem::swap(&mut child.children, &mut leftover_child.children);
        }
        true
    }

    #[cfg(test)]
    pub(crate) fn delete(&mut self, word: &str) {
        // No need of doing anything if the word is empty
        if word.is_empty() {
            return;
        }
        // Mutable pointer to switch between the parents and children
        let mut parent: &mut PatriciaNode = self;
        // Clone to avoid destroying given data
        let mut word_cpy = word.to_string();

        loop {
            let index_child: usize;

            let res = parent.children.binary_search_by(|child| {
                child.letters.chars().next().cmp(&word_cpy.chars().next())
            });

            match res {
                Ok(r) => {
                    index_child = r;
                    // If node not deleted then we must continue looping
                    if !parent.delete_node(&word_cpy, r) {
                        let child = parent.children.get_mut(r).unwrap();
                        word_cpy = word_cpy.split_off(child.letters.len());
                    }
                    // Stop condition
                    else {
                        return;
                    }
                }
                Err(_) => {
                    return;
                }
            }
            // Switch between parent and child
            parent = parent.children.get_mut(index_child).unwrap();
        }
    }

    /// Recursive search in patricia trie of a word
    pub(crate) fn search(&self, mut word: String) -> Option<&Self> {
        if self.children.is_empty() {
            None
        } else {
            let res = self
                .children
                .binary_search_by(|child| child.letters.chars().next().cmp(&word.chars().next()));

            match res {
                Ok(r) => {
                    let child = self.children.get(r).unwrap();
                    let word_cpy: String;

                    match child.letters.len().cmp(&word.len()) {
                        std::cmp::Ordering::Greater => None,
                        std::cmp::Ordering::Equal => {
                            if child.letters != word {
                                None
                            } else {
                                Some(child)
                            }
                        }
                        _ => {
                            word_cpy = word.split_off(child.letters.len());
                            if child.letters != word {
                                None
                            } else {
                                word = word_cpy;
                                child.search(word)
                            }
                        }
                    }
                }
                Err(_) => None,
            }
        }
    }
}

impl TrieNodeDrainer for PatriciaNode {
    fn drain_characters(&mut self) -> String {
        std::mem::replace(&mut self.letters, String::new())
    }

    fn frequency(&self) -> Option<NonZeroU32> {
        self.freq
    }

    fn drain_children(&mut self) -> Vec<Self> {
        std::mem::replace(&mut self.children, Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_patricia() -> PatriciaNode {
        PatriciaNode {
            letters: String::new(),
            children: Vec::new(),
            freq: None,
        }
    }

    #[test]
    fn empty_creation() {
        let mut parent = empty_patricia();
        parent.insert(&String::new(), NonZeroU32::new(1).unwrap());
        assert!(parent.children.is_empty())
    }

    #[test]
    fn insert_one_word() {
        // Create parent and insert a child
        let mut parent = empty_patricia();
        parent.insert(&String::from("abc"), NonZeroU32::new(1).unwrap());

        // Create expected result
        let expected_node = PatriciaNode {
            letters: String::from("abc"),
            children: Vec::new(),
            freq: NonZeroU32::new(1),
        };
        let mut expected = Vec::new();
        expected.push(expected_node);

        println!("{:?}", parent);
        // Compare
        assert!(parent.children.len() == 1);
        assert!(parent.freq == None);
        assert_eq!(parent.children, expected)
    }

    #[test]
    fn insert_multiple_different_words() {
        let mut parent = empty_patricia();
        let default_freq = 1;

        parent.insert(&String::from("abc"), NonZeroU32::new(default_freq).unwrap());
        parent.insert(&String::from("cab"), NonZeroU32::new(default_freq).unwrap());
        parent.insert(&String::from("bac"), NonZeroU32::new(default_freq).unwrap());

        let expected_abc = PatriciaNode {
            letters: String::from("abc"),
            children: Vec::new(),
            freq: NonZeroU32::new(default_freq),
        };
        let expected_bac = PatriciaNode {
            letters: String::from("bac"),
            children: Vec::new(),
            freq: NonZeroU32::new(default_freq),
        };
        let expected_cab = PatriciaNode {
            letters: String::from("cab"),
            children: Vec::new(),
            freq: NonZeroU32::new(default_freq),
        };
        let expected = vec![expected_abc, expected_bac, expected_cab];

        assert!(parent.children.len() == 3);
        assert!(parent.freq == None);
        assert_eq!(parent.children, expected)
    }

    #[test]
    fn insert_continuation_word() {
        let mut parent = empty_patricia();
        let default_freq = 1;

        parent.insert(&String::from("abc"), NonZeroU32::new(default_freq).unwrap());
        parent.insert(
            &String::from("abcdefg"),
            NonZeroU32::new(default_freq).unwrap(),
        );

        assert!(parent.children.len() == 1);
        let only_child = parent.children.pop().unwrap();

        let expected_defg = PatriciaNode {
            letters: String::from("defg"),
            children: Vec::new(),
            freq: NonZeroU32::new(default_freq),
        };
        let expected_abc = PatriciaNode {
            letters: String::from("abc"),
            children: vec![expected_defg],
            freq: NonZeroU32::new(default_freq),
        };

        assert_eq!(only_child, expected_abc);
    }

    #[test]
    fn insert_in_already_word() {
        let mut parent = empty_patricia();
        let default_freq = 1;

        parent.insert(
            &String::from("abcdefg"),
            NonZeroU32::new(default_freq).unwrap(),
        );
        parent.insert(&String::from("abc"), NonZeroU32::new(2).unwrap());
        parent.insert(
            &String::from("abcklm"),
            NonZeroU32::new(default_freq).unwrap(),
        );

        assert!(parent.children.len() == 1);
        // abc
        let only_child = parent.children.pop().unwrap();

        let expected_defg = PatriciaNode {
            letters: String::from("defg"),
            children: Vec::new(),
            freq: NonZeroU32::new(default_freq),
        };
        let expected_klm = PatriciaNode {
            letters: String::from("klm"),
            children: Vec::new(),
            freq: NonZeroU32::new(default_freq),
        };
        let expected_abc = PatriciaNode {
            letters: String::from("abc"),
            children: vec![expected_defg, expected_klm],
            freq: NonZeroU32::new(2),
        };

        assert!(only_child.children.len() == 2);
        assert_eq!(only_child.children, expected_abc.children);
    }

    #[test]
    fn simple_insert_delete() {
        let mut parent = empty_patricia();
        let default_freq = 1;

        parent.insert(&String::from("abc"), NonZeroU32::new(default_freq).unwrap());

        assert!(parent.children.len() == 1);

        parent.delete(&String::from("abc"));
        println!("{:?}", parent);

        assert!(parent.children.is_empty());
    }

    #[test]
    fn multiple_insert_inner_delete() {
        let mut parent = empty_patricia();
        let default_freq = 1;

        parent.insert(
            &String::from("abcdefg"),
            NonZeroU32::new(default_freq).unwrap(),
        );
        parent.insert(&String::from("abc"), NonZeroU32::new(2).unwrap());
        parent.insert(
            &String::from("abcklm"),
            NonZeroU32::new(default_freq).unwrap(),
        );

        parent.delete(&String::from("abc"));

        assert!(parent.children.len() == 1);
        let only_child = parent.children.pop().unwrap();
        assert!(only_child.children.len() == 2);
        assert!(only_child.freq == None);
    }

    #[test]
    fn delete_combination() {
        let mut parent = empty_patricia();
        let default_freq = 1;

        parent.insert(
            &String::from("abcdefg"),
            NonZeroU32::new(default_freq).unwrap(),
        );
        parent.insert(&String::from("abc"), NonZeroU32::new(2).unwrap());

        parent.delete(&String::from("abc"));

        assert!(parent.children.len() == 1);
        let only_child = parent.children.pop().unwrap();
        assert_eq!(only_child.letters, "abcdefg");
        assert!(only_child.children.is_empty());
    }

    #[test]
    fn delete_not_existing() {
        let mut parent = empty_patricia();
        let default_freq = 1;

        parent.insert(
            &String::from("abcdefg"),
            NonZeroU32::new(default_freq).unwrap(),
        );
        parent.insert(&String::from("abc"), NonZeroU32::new(2).unwrap());
        parent.insert(
            &String::from("abcklm"),
            NonZeroU32::new(default_freq).unwrap(),
        );

        let parent_clone = parent.clone();

        parent.delete(&String::from("ab"));
        parent.delete(&String::from("abck"));
        parent.delete(&String::from("abcdefgk"));

        assert_eq!(parent, parent_clone);
    }

    #[test]
    fn simple_search() {
        let mut parent = empty_patricia();

        parent.insert(&String::from("abc"), NonZeroU32::new(2).unwrap());

        let child = parent.search(String::from("abc"));
        assert!(child.is_some());

        let expected_child = PatriciaNode {
            letters: String::from("abc"),
            children: Vec::new(),
            freq: NonZeroU32::new(2),
        };

        assert_eq!(child.unwrap(), &expected_child);
    }

    #[test]
    fn inner_search() {
        let mut parent = empty_patricia();

        parent.insert(&String::from("abc"), NonZeroU32::new(2).unwrap());
        parent.insert(&String::from("abcdefg"), NonZeroU32::new(1).unwrap());

        let child = parent.search(String::from("abcdefg"));
        assert!(child.is_some());

        let expected_child = PatriciaNode {
            letters: String::from("defg"),
            children: Vec::new(),
            freq: NonZeroU32::new(1),
        };
        assert_eq!(child.unwrap(), &expected_child);
    }
}
