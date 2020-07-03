use std::num::NonZeroU32;
#[derive(Debug, Clone)]
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

    pub(crate) fn insert(&mut self, word: &String, frequency: NonZeroU32) {
        let mut parent: &mut PatriciaNode= self;

        loop {
            let mut index_child: usize = 0;

            let res = parent.children.binary_search_by(|child| 
                child.letters.chars().next().cmp(&word.chars().next()));

            let inserted = match res {
                Ok(r) => {
                    let child = parent.children.get_mut(r).unwrap();
                    let insrt = child.divide(word, frequency);
                    if !insrt {
                        index_child = r;
                    }
                    insrt
                }
                Err(r) => { 
                    parent.createAndInsertAt(r, word, frequency);
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