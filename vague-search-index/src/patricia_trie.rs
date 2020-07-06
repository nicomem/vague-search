use crate::error::*;
use crate::utils::read_lines;
use snafu::*;
use std::num::NonZeroU32;
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

    pub(crate) fn create_from_file(filepath: &str) -> Result<Self> {
        let mut root = Self::create_empty();
        let lines = read_lines(filepath).context(FileOpen { path: filepath })?;
        for (line_num, line) in lines.enumerate() {
            let wordfreq = line.context(FileRead { path: filepath })?;
            let mut iter = wordfreq.split_whitespace();
            // Parse word
            let word = iter.next().context(ContentRead {
                path: filepath,
                line: &wordfreq,
                number: line_num,
            })?;

            // Parse frequency
            let freqstr = iter.next().context(ContentRead {
                path: filepath,
                line: &wordfreq,
                number: line_num,
            })?;
            let freq = freqstr.parse::<NonZeroU32>().context(Parsing {
                path: filepath,
                number: line_num,
            })?;
            root.insert(word, freq)
        }
        Ok(root)
    }

    ///  Divides a node by two in indicated index and creates the childs accordingly
    fn divide_node(&mut self, word: &str, ind: usize, frequency: NonZeroU32) {
        let second_part = self.letters.split_off(ind);

        let mut new_node = PatriciaNode {
            letters: second_part,
            children: Vec::new(),
            freq: self.freq,
        };
        // Swap children
        std::mem::swap(&mut self.children, &mut new_node.children);

        let (_, second_word) = word.split_at(ind);
        self.children = vec![new_node];
        self.freq = None;
        // Split off already changed letters

        // push node only if word to insert isn't empty
        if !second_word.is_empty() {
            let new_word_node = PatriciaNode {
                letters: second_word.to_string(),
                children: Vec::new(),
                freq: Some(frequency),
            };
            self.children.push(new_word_node);
        }
        // otherwise current node is a word, add the frequency
        else {
            self.freq = Some(frequency);
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
    pub(crate) fn insert(&mut self, word: &str, frequency: NonZeroU32) {
        // No need of doing anything if the word is empty
        if word.is_empty() {
            return;
        }

        // Mutable pointer to switch between the parents and children
        let mut parent: &mut PatriciaNode = self;
        // Clone to avoid destroying given data
        let mut word_cpy = word.to_string();

        loop {
            let mut index_child: usize = 0;

            let res = parent.children.binary_search_by(|child| {
                child.letters.chars().next().cmp(&word_cpy.chars().next())
            });

            let inserted = match res {
                Ok(r) => {
                    let child = parent.children.get_mut(r).unwrap();
                    let insrt = child.divide(&word_cpy, frequency);
                    if !insrt {
                        index_child = r;
                        word_cpy = word_cpy.split_off(child.letters.len());
                    }
                    insrt
                }
                Err(r) => {
                    parent.create_and_insert_at(r, &word_cpy, frequency);
                    true
                }
            };
            if inserted {
                break;
            }

            parent = parent.children.get_mut(index_child).unwrap()
        }
    }

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
