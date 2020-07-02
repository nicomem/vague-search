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
        else {
            self.freq = Some(frequency);
        }
    }

    pub(crate) fn insert(&mut self, word: &String, frequency: NonZeroU32) {
        let mut parent: &mut PatriciaNode= self;

        // otherwise we must loop on the potential cases
        loop {
            let mut index_diff = Some(0);
            let mut index_child: usize = 0;

            for child in &mut parent.children {
                index_diff = indexDifference(&child.letters, &word);
                match index_diff {
                    Some(ind) => {
                        if ind != child.letters.len() && ind != 0 {
                            child.divideNode(&word, ind, frequency);
                            return
                        }
                        else if ind != 0{
                            break;
                        }
                    }
                    None => { break; }
                }
                index_child += 1;
            }
            if index_diff.is_some() && index_diff.unwrap() == 0 {
                let child = PatriciaNode{letters: word.clone(), children: Vec::new(), freq: None};
                parent.children.push(child);
                break;
            }
            
            parent = parent.children.get_mut(index_child).unwrap()
        }
        
    }
}