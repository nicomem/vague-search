// TODO Add search approx function

use crate::layer_stack::LayerStack;
use std::{cmp::min, num::NonZeroU32};
use vague_search_core::{CompiledTrie, CompiledTrieNode, NodeValue};

/// A type to store searching distances.
pub type Distance = u16;

/// A type to store word sizes.
pub type WordSize = u16;

/// A stack of iterations, used to linearise the recursive searching algorithm.
pub type IterationStack = Vec<Option<CompiledTrieNode>>; // TODO

/// A word that have been found by a search query.
#[derive(Eq, PartialEq)]
pub struct FoundWord {
    pub word: String,
    pub freq: NonZeroU32,
    pub dist: Distance,
}

impl PartialOrd for FoundWord {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FoundWord {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.dist
            .cmp(&other.dist)
            .then(self.freq.cmp(&other.freq))
            .then(self.word.cmp(&other.word))
    }
}

fn init_array(num: usize) -> Vec<u32> {
    let vector = Vec::with_capacity(num);
    for i in 0..(num as u32) {
        vector.push(i);
    }
    vector
}

/// Levenshtein updating of matrices lines
fn update_line(new_line: &mut [u16], parent_line: &[u16], same_letters: bool) {
    for (i, cell) in new_line.iter_mut().enumerate() {
        let insert_cost = new_line[i - 1] + 1;
        let delete_cost = parent_line[i];
        let replace_cost = parent_line[i] + (same_letters as u16);

        // TODO: Compute Damerau substitution cost
        let subst_cost = u16::MAX;
        *cell = min(min(insert_cost, delete_cost), min(replace_cost, subst_cost));
    }
}

/// Returns the current distance taking only tested letters
/// FIXME: Maybe add len checking in the process (word_len >= curr_len)
fn current_distance(line: &[u16], curr_len: WordSize) -> u16 {
    line[curr_len as usize]
}

/// Returns the best distance taking into account the whole words
/// FIXME: Maybe add len checking in the process
fn full_distance(line: &[u16], word_len: WordSize) -> u16 {
    line[word_len as usize]
}

/// Search for all words in the trie at a given distance (or less) of the query.
///
/// Return a vector of all found words with their respective frequency.
pub fn search_approx(
    trie: &CompiledTrie,
    word: &str,
    distance: Distance,
    layer_stack: &mut LayerStack<Distance, WordSize>,
    iter_stack: &mut IterationStack,
    result_buffer: Vec<FoundWord>,
) -> Vec<FoundWord> {
    // Early return if nothing to search
    if word.is_empty() {
        return result_buffer;
    }

    let roots = trie.get_root_siblings().unwrap();

    // Keep track of current recursive word length in trie
    let trie_sizes_stack: Vec<u16> = Vec::new();
    // First row begins by 0
    trie_sizes_stack.push(0);

    let root_line = init_array(word.len() + 1);
    for root in roots {
        iter_stack.push(Some(*root));
        loop {
            let compiled_node_option = match iter_stack.pop() {
                Some(r) => r,
                None => break,
            };

            // If layer found: continue
            // Else loop is over
            let compiled_node = match compiled_node_option {
                Some(node) => node,
                None => {
                    if !layer_stack.pop_layer() || trie_sizes_stack.pop().is_none() {
                        break;
                    }
                    continue;
                }
            };

            // Fetch lines
            let parent_line = layer_stack.fetch_layer().unwrap();
            let current_length = trie_sizes_stack.last().unwrap();
            let new_line = layer_stack.push_layer(word.len() as WordSize + 1);

            // Compute line
            match compiled_node.node_value() {
                NodeValue::Naive(node) => {
                    // init first index with current length
                    new_line[0] = *current_length;
                    // potential FIXME
                    update_line(
                        new_line,
                        parent_line,
                        node.character == word.chars().next().unwrap(),
                    );

                    if let Some(index) = node.index_first_child {
                        iter_stack.push(None);
                        for child in trie.get_siblings(index).iter().rev() {
                            // Push children
                            iter_stack.push(Some(*child));
                        }
                    }
                }
                NodeValue::Patricia(node) => {
                    let chars_it = trie
                        // SAFETY: Safe because in a patricia node
                        .get_chars(unsafe { &compiled_node.patricia_range() })
                        .chars();
                    let word_it = word.chars();

                    // Calculate and update lines
                    // Also update curr_length
                    // The length will allow to check if one of the word ended first
                    let length: WordSize = 0;
                    // FIXME abort if current distance greater than minimum distance
                    loop {
                        let node_char = chars_it.next();
                        let word_char = word_it.next();

                        // Word finished before patricia finished
                        // This wasn't a potential node
                        if node_char.is_some() && word_char.is_none() {
                            return None; // TODO: Don't return but continue queue
                        }

                        if node_char.is_none() || word_char.is_none() {
                            break;
                        }

                        new_line[0] = *current_length;
                        update_line(
                            new_line,
                            parent_line,
                            node_char.unwrap() == word_char.unwrap(),
                        );
                        // Copy from NodeLine (equivalent to clone), sizes are assured to be the same
                        // Allows to reuse parent_line for next iteration and node_line calculation
                        parent_line.copy_from_slice(new_line);
                        length += 1;
                    }
                    if word.len() == (current_length + length) as usize && node.word_freq.is_some()
                    {
                        result_buffer.push(FoundWord {
                            word: String::new(),
                            freq: node.word_freq.unwrap(),
                            dist: 0,
                        })
                    }

                    if let Some(index) = node.index_first_child {
                        iter_stack.push(None);
                        for child in trie.get_siblings(index).iter().rev() {
                            iter_stack.push(Some(*child));
                        }
                    }
                }
                NodeValue::Range(node) => {
                    let ranges = trie.get_range(&(node.start_index..node.end_index));
                }
            }

            // Find children
        }
    }

    result_buffer
}
