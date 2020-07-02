use std::num::NonZeroU32;
#[derive(Debug, Clone)]
pub struct PatriciaNode {
    letters: String,
    children: Vec<PatriciaNode>,
    pub freq: Option<NonZeroU32>,
}

pub fn indexDifference(first: &String, second: &String) -> u8
{
    let mut index: u8 = 0;
    for (ai, bi) in first.chars().zip(second.chars()){
        if ai != bi {
            break;
        }
        index += 1;
    }
    index
}

impl PatriciaNode {
    /**
     * Divides a node by two in indicated index and creates the childs accordingly
     */
    fn divideNode(mut self, word: &String, ind: u8, frequency: NonZeroU32)
    {
        let (first_part, second_part) = self.letters.split_at(ind as usize);
        let new_node = PatriciaNode {letters: second_part.to_string(), children: self.children, freq: self.freq};
        let (_, second_word) = word.split_at(ind as usize);
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

    pub(crate) fn insert(&mut self, word: String, frequency: NonZeroU32) {
        let mut parent: &mut PatriciaNode= self;
        let word_cpy = word;

        if parent.children.is_empty() {
            let node = PatriciaNode{letters: word_cpy, children: Vec::new(), freq: None};
            parent.children.push(node);
            return;
        }
        let mut node = None;
        loop {
            node = None;
            let mut ind = 0;
            for child in &mut parent.children {
                ind = indexDifference(&child.letters, &word_cpy);
                if ind != 0 {
                    node = Some(child);
                    break
                }
            }
            if let Some(n) = node {
                // Divide current node in two and insert the rest of the word if not empty
                    //let mut actual_node = node.unwrap().to_owned();
                    if n.letters.len() != ind as usize {
                        n.divideNode(&word_cpy, ind, frequency);
                        break;
                    }
            
                    // Switch parent node if word is fully matched for the next iteration
                    parent = n;
            }
            else {
                node = None;
                let child = PatriciaNode{letters: word_cpy, children: Vec::new(), freq: None};
                parent.children.push(child);
                break
            }
        }
        
    }
}

/*
For children:
 parcours word, if first letter does not match do not use
 if first letter match till last letter or till it does
 not match anymore
 if none match create a new 
*/