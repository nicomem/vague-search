use std::num::NonZeroU32;
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PatriciaNode {
    letters: String,
    children: Vec<PatriciaNode>,
    freq: Option<NonZeroU32>,
}

pub fn indexDifference(first: &String, second: &String) -> Option<usize>
{
    first.chars().zip(second.chars()).position(|(a, b)| a != b)
}

impl PatriciaNode {
    
    ///  Divides a node by two in indicated index and creates the childs accordingly
    fn divideNode(&mut self, word: &String, ind: usize, frequency: NonZeroU32)
    {
        let (first_part, second_part) = self.letters.split_at(ind);
        
        let mut new_node = PatriciaNode {letters: second_part.to_string(), children: Vec::new(), freq: self.freq};
        // Swap children
        std::mem::swap(&mut self.children, &mut new_node.children);

        let (_, second_word) = word.split_at(ind);
        self.children = vec![new_node];
        self.freq = None;
        self.letters = first_part.to_string();

        // push node only if word to insert isn't empty
        if !second_word.is_empty() {
            let new_word_node = PatriciaNode {letters: second_word.to_string(), children: Vec::new(), freq: Some(frequency)};
            self.children.push(new_word_node);
        }
        // otherwise current node is a word, add the frequency
        else {
            self.freq = Some(frequency);
        }
    }

    fn createAndInsertAt(&mut self, index: usize, word: &String, frequency: NonZeroU32){
        let child = PatriciaNode{letters: word.clone(), children: Vec::new(), freq: Some(frequency)};
        self.children.insert(index, child);

    }

    fn divide(&mut self, word: &String, frequency: NonZeroU32) -> bool
    {
        let index_diff = indexDifference(&self.letters, &word);
        match index_diff {
            Some(ind) => {
                self.divideNode(word, ind, frequency)
            }
            None => {
                if word.len() < self.letters.len() {
                    self.divideNode(word, word.len(), frequency)
                }
                else if word.len() == self.letters.len() {
                    self.freq = Some(frequency);
                }
                else {
                    return false;
                }
            }
        }
        return true;
    }

    /// Insert a word and its frequency in the patricia trie
    pub(crate) fn insert(&mut self, word: &String, frequency: NonZeroU32) {
        // Mutable pointer to switch between the parents and children
        let mut parent: &mut PatriciaNode= self;
        // Clone to avoid destroying given data
        let mut word_cpy = word.clone();

        // No need of doing anything if the word is empty
        if word.is_empty() {
            return;
        }

        loop {
            let mut index_child: usize = 0;

            let res = parent.children.binary_search_by(|child| 
                child.letters.chars().next().cmp(&word_cpy.chars().next()));

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
                    parent.createAndInsertAt(r, &word_cpy, frequency);
                    true
                }
            };
            if inserted {
                break;
            }
            
            parent = parent.children.get_mut(index_child).unwrap()
        }
        
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    fn empty_patricia() -> PatriciaNode {
        PatriciaNode {letters: String::new(), children: Vec::new(), freq: None}
    }

    #[test]
    fn empty_creation() {
        let mut parent = empty_patricia();
        parent.insert(&String::new(), NonZeroU32::new(1).unwrap());
        assert!(parent.children.is_empty())
    }

    #[test]
    fn insert_one_word () {
        // Create parent and insert a child
        let mut parent = empty_patricia();
        parent.insert(&String::from("abc"), NonZeroU32::new(1).unwrap());

        // Create expected result
        let expected_node = PatriciaNode {letters: String::from("abc"), children: Vec::new(), freq: NonZeroU32::new(1)};
        let mut expected = Vec::new();
        expected.push(expected_node);

        // Compare
        assert!(parent.children.len() == 1);
        assert!(parent.freq == None);
        assert_eq!(parent.children, expected)
    }

    #[test]
    fn insert_multiple_different_words () {
        let mut parent = empty_patricia();
        let default_freq  = 1;

        parent.insert(&String::from("abc"), NonZeroU32::new(default_freq).unwrap());
        parent.insert(&String::from("cab"), NonZeroU32::new(default_freq).unwrap());
        parent.insert(&String::from("bac"), NonZeroU32::new(default_freq).unwrap());

        let expected_abc = PatriciaNode {letters: String::from("abc"), children: Vec::new(), freq: NonZeroU32::new(default_freq)};
        let expected_bac = PatriciaNode {letters: String::from("bac"), children: Vec::new(), freq: NonZeroU32::new(default_freq)};
        let expected_cab = PatriciaNode {letters: String::from("cab"), children: Vec::new(), freq: NonZeroU32::new(default_freq)};
        let expected = vec![expected_abc, expected_bac, expected_cab];

        assert!(parent.children.len() == 3);
        assert!(parent.freq == None);
        assert_eq!(parent.children, expected)
    }

    #[test]
    fn insert_continuation_word () {
        let mut parent = empty_patricia();
        let default_freq = 1;

        parent.insert(&String::from("abc"), NonZeroU32::new(default_freq).unwrap());
        parent.insert(&String::from("abcdefg"), NonZeroU32::new(default_freq).unwrap());

        assert!(parent.children.len() == 1);
        let only_child = parent.children.pop().unwrap();
        
        let expected_defg = PatriciaNode {letters: String::from("defg"), children: Vec::new(), freq: NonZeroU32::new(default_freq)};
        let expected_abc = PatriciaNode {letters: String::from("abc"), children: vec![expected_defg], freq: NonZeroU32::new(default_freq)};

        assert_eq!(only_child, expected_abc);
    }

    #[test]
    fn insert_in_already_word () {
        let mut parent = empty_patricia();
        let default_freq = 1;

        parent.insert(&String::from("abcdefg"), NonZeroU32::new(default_freq).unwrap());
        parent.insert(&String::from("abc"), NonZeroU32::new(2).unwrap());
        parent.insert(&String::from("abcklm"), NonZeroU32::new(default_freq).unwrap());

        assert!(parent.children.len() == 1);
        // abc
        let only_child = parent.children.pop().unwrap();

        let expected_defg = PatriciaNode {letters: String::from("defg"), children: Vec::new(), freq: NonZeroU32::new(default_freq)};
        let expected_klm = PatriciaNode {letters: String::from("klm"), children: Vec::new(), freq: NonZeroU32::new(default_freq)};
        let expected_abc = PatriciaNode {letters: String::from("abc"), children: vec![expected_defg, expected_klm], freq: NonZeroU32::new(2)};

        assert!(only_child.children.len() == 2);
        assert_eq!(only_child.children, expected_abc.children);
    }


}