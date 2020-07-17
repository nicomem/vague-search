use crate::layer_stack::LayerStack;
use std::{cmp::min, num::NonZeroU32};
use vague_search_core::{
    CompiledTrie, CompiledTrieNode, NaiveNode, NodeValue, PatriciaNode, RangeNode,
};

/// A type to store searching distances.
pub type Distance = u16;

/// A type to store word sizes.
pub type WordSize = u16;

/// An iteration element. Includes what is needed to continue the iteration.
/// Similar to what the compiler would store during a recursion call.
/// However by doing it manually, some optimizations can be applied.
pub struct IterationElement<'a> {
    /// The current trie node.
    node: &'a CompiledTrieNode,

    /// The last character of the trie path, useful for the Damerau-Levenshtein
    /// distance computation.
    last_char: Option<char>,

    /// The current index in the range.
    /// Its value is not specified if the node is not a [RangeNode](vague_search_core::RangeNode).
    range_offset: u32,
}

/// A stack of iterations, used to linearise the recursive searching algorithm.
/// The end of a layer is represented by a "dummy node", which is a None element.
pub type IterationStack<'a> = Vec<Option<IterationElement<'a>>>;

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

/// Retrieve and push the root nodes in the iteration stack.
fn push_layer_nodes<'a>(iter_stack: &mut IterationStack<'a>, nodes: &'a [CompiledTrieNode]) {
    iter_stack.reserve(nodes.len() + 1);

    // Push a dummy node to represent the end of the layer
    iter_stack.push(None);

    // Push the nodes in reverse order to pop them in the correct order in the future
    for node in nodes.iter().rev() {
        iter_stack.push(Some(IterationElement {
            node,
            last_char: None,
            range_offset: 0,
        }));
    }
}

/// Create and push the first layer in the layer stack.
fn push_first_layer(
    layer_stack: &mut LayerStack<Distance, WordSize>,
    layer_char: Option<char>,
    word_size: WordSize,
) {
    let layer = layer_stack.push_layer(layer_char, word_size);
    for (i, e) in layer.iter_mut().enumerate() {
        *e = i as _;
    }
}

fn push_layers_naive(
    node: &NaiveNode,
    iter_elem: &IterationElement,
    word: &str,
    layer_stack: &mut LayerStack<Distance, WordSize>,
) {
    let layer = layer_stack.push_layer(Some(node.character), word.len() as _);

    todo!()
}

fn push_layers_patricia(
    node: &PatriciaNode,
    iter_elem: &IterationElement,
    word: &str,
    layer_stack: &mut LayerStack<Distance, WordSize>,
    trie: &CompiledTrie,
) {
    // SAFETY: Safe because in a patricia node
    let range_chars = unsafe { iter_elem.node.patricia_range() };
    let pat_chars = trie.get_chars(&range_chars);

    for ch in pat_chars.chars() {
        let layer = layer_stack.push_layer(Some(ch), word.len() as _);
        todo!("Should be like naive but with looping through the pat characters")
    }
}

fn push_layers_range(
    node: &RangeNode,
    iter_elem: &IterationElement,
    word: &str,
    layer_stack: &mut LayerStack<Distance, WordSize>,
) {
    // SAFETY: Safety checked during dictionary compilation
    let character =
        unsafe { std::char::from_u32_unchecked(node.first_char as u32 + iter_elem.range_offset) };
    let layer = layer_stack.push_layer(Some(character), word.len() as _);

    todo!()
}

/// Process the current node and update the layer stack with the node's new layers.
fn push_layers_current_node(
    iter_elem: &IterationElement,
    word: &str,
    trie: &CompiledTrie,
    layer_stack: &mut LayerStack<Distance, WordSize>,
) {
    match iter_elem.node.node_value() {
        NodeValue::Naive(n) => push_layers_naive(n, iter_elem, word, layer_stack),
        NodeValue::Patricia(n) => push_layers_patricia(n, iter_elem, word, layer_stack, trie),
        NodeValue::Range(n) => push_layers_range(n, iter_elem, word, layer_stack),
    }
}

/// Retrieve the node frequency.
fn get_node_frequency(iter_elem: &IterationElement, trie: &CompiledTrie) -> Option<NonZeroU32> {
    match iter_elem.node.node_value() {
        NodeValue::Naive(n) => n.word_freq,
        NodeValue::Patricia(n) => n.word_freq,
        NodeValue::Range(n) => {
            let range = n.start_index..n.end_index;
            let slice = trie.get_range(&range);
            let elem = &slice[iter_elem.range_offset as usize];
            elem.word_freq
        }
    }
}

/// Get the current distance to the query word from the current distance layer.
fn get_current_distance(cur_layer: &[Distance]) -> Distance {
    *cur_layer.last().unwrap()
}

/// Check if the word can be added to the result and add it if so.
fn check_add_word_to_result(
    iter_elem: &IterationElement,
    cur_layer: &[Distance],
    dist_max: Distance,
    layer_word: &str,
    trie: &CompiledTrie,
    result_buffer: &mut Vec<FoundWord>,
) {
    // If end word and less than max dist => Add to result
    let dist = get_current_distance(cur_layer);
    if dist <= dist_max {
        if let Some(freq) = get_node_frequency(iter_elem, trie) {
            result_buffer.push(FoundWord {
                word: layer_word.to_owned(),
                freq,
                dist,
            })
        }
    }
}

fn any_below_max_dist(cur_layer: &[Distance], dist_max: Distance) -> bool {
    cur_layer.iter().any(|&d| d < dist_max)
}

fn get_node_children<'a>(
    trie: &'a CompiledTrie,
    iter_elem: &IterationElement,
) -> &'a [CompiledTrieNode] {
    // Get the index of the node's first child
    let index = match iter_elem.node.node_value() {
        NodeValue::Naive(n) => n.index_first_child,
        NodeValue::Patricia(n) => n.index_first_child,
        NodeValue::Range(n) => {
            let range = n.start_index..n.end_index;
            let slice = trie.get_range(&range);
            let elem = &slice[iter_elem.range_offset as usize];
            elem.index_first_child
        }
    };

    // Get it and its siblings, or return an empty slice if no index (no children)
    match index {
        Some(i) => trie.get_siblings(i),
        None => Default::default(),
    }
}

/// Search for all words in the trie at a given distance (or less) of the query.
///
/// Return a vector of all found words with their respective frequency.
pub fn search_approx<'a>(
    trie: &'a CompiledTrie,
    word: &str,
    dist_max: Distance,
    layer_stack: &mut LayerStack<Distance, WordSize>,
    iter_stack: &mut IterationStack<'a>,
    mut result_buffer: Vec<FoundWord>,
) -> Vec<FoundWord> {
    // Early return if nothing to search
    if word.is_empty() {
        return result_buffer;
    }

    // Retrieve the root nodes
    let roots = trie.get_root_siblings().unwrap();

    // Initialize both stacks
    push_layer_nodes(iter_stack, roots);
    push_first_layer(layer_stack, None, word.len() as _);

    // Loop over the iteration stack until empty
    while let Some(iter_elem_opt) = iter_stack.pop() {
        // Extract the node or process the dummy node
        let iter_elem = match iter_elem_opt {
            Some(n) => n,
            None => {
                // Dummy node => represents the end of a layer
                layer_stack.pop_layer();
                continue;
            }
        };

        // Compute and push the distance layers of the current node
        push_layers_current_node(&iter_elem, word, trie, layer_stack);

        // SAFETY: The layer stack is not empty at this point
        let cur_layer = layer_stack
            .fetch_layer()
            .unwrap_or_else(|| unsafe { std::hint::unreachable_unchecked() });

        let layer_word = layer_stack.get_layers_word();

        // Add trie node's word to result if it can be
        check_add_word_to_result(
            &iter_elem,
            cur_layer,
            dist_max,
            layer_word,
            trie,
            &mut result_buffer,
        );

        let children = get_node_children(trie, &iter_elem);

        // If no children or current word has exceeded dist_max,
        // remove its layer and continue with next iteration
        if children.is_empty() || !any_below_max_dist(cur_layer, dist_max) {
            layer_stack.pop_layer();
            continue;
        }

        // Add all children to the stack
        push_layer_nodes(iter_stack, children);
    }

    // Return the result buffer that has been filled in the stack loop
    result_buffer
}

/// Levenshtein updating of matrices lines
fn update_line(new_line: &mut [u16], parent_line: &[u16], same_letters: bool) {
    for i in 1..new_line.len() {
        let insert_cost = new_line[i - 1] + 1;
        let delete_cost = parent_line[i];
        let replace_cost = parent_line[i] + (same_letters as u16);

        // TODO: Compute Damerau substitution cost
        let subst_cost = u16::MAX;
        new_line[i] = min(min(insert_cost, delete_cost), min(replace_cost, subst_cost));
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

// // Keep track of current recursive word length in trie
// let trie_sizes_stack: Vec<u16> = Vec::new();
// // First row begins by 0
// trie_sizes_stack.push(0);

// let root_line = init_array(word.len() + 1);
// for root in roots {
//     iter_stack.push(Some(*root));
//     loop {
//         let compiled_node_option = match iter_stack.pop() {
//             Some(r) => r,
//             None => break,
//         };

//         // If layer found: continue
//         // Else loop is over
//         let compiled_node = match compiled_node_option {
//             Some(node) => node,
//             None => {
//                 if !layer_stack.pop_layer() || trie_sizes_stack.pop().is_none() {
//                     break;
//                 }
//                 continue;
//             }
//         };

//         // Fetch lines
//         let parent_line = layer_stack.fetch_layer().unwrap();
//         let current_length = trie_sizes_stack.last().unwrap();
//         let new_line = layer_stack.push_layer(word.len() as WordSize + 1);

//         // Compute line
//         match compiled_node.node_value() {
//             NodeValue::Naive(node) => {
//                 // init first index with current length
//                 new_line[0] = *current_length;
//                 // potential FIXME
//                 update_line(
//                     new_line,
//                     parent_line,
//                     node.character == word.chars().next().unwrap(),
//                 );

//                 if let Some(index) = node.index_first_child {
//                     iter_stack.push(None);
//                     for child in trie.get_siblings(index).iter().rev() {
//                         // Push children
//                         iter_stack.push(Some(*child));
//                     }
//                 }
//             }
//             NodeValue::Patricia(node) => {
//                 let chars_it = trie
//                     // SAFETY: Safe because in a patricia node
//                     .get_chars(unsafe { &compiled_node.patricia_range() })
//                     .chars();
//                 let word_it = word.chars();

//                 // Calculate and update lines
//                 // Also update curr_length
//                 // The length will allow to check if one of the word ended first
//                 let length: WordSize = 0;
//                 // FIXME abort if current distance greater than minimum distance
//                 loop {
//                     let node_char = chars_it.next();
//                     let word_char = word_it.next();

//                     // Word finished before patricia finished
//                     // This wasn't a potential node
//                     if node_char.is_some() && word_char.is_none() {
//                         return None; // TODO: Don't return but continue queue
//                     }

//                     if node_char.is_none() || word_char.is_none() {
//                         break;
//                     }

//                     new_line[0] = *current_length;
//                     update_line(
//                         new_line,
//                         parent_line,
//                         node_char.unwrap() == word_char.unwrap(),
//                     );
//                     // Copy from NodeLine (equivalent to clone), sizes are assured to be the same
//                     // Allows to reuse parent_line for next iteration and node_line calculation
//                     parent_line.copy_from_slice(new_line);
//                     length += 1;
//                 }
//                 if word.len() == (current_length + length) as usize && node.word_freq.is_some()
//                 {
//                     result_buffer.push(FoundWord {
//                         word: String::new(),
//                         freq: node.word_freq.unwrap(),
//                         dist: 0,
//                     })
//                 }

//                 if let Some(index) = node.index_first_child {
//                     iter_stack.push(None);
//                     for child in trie.get_siblings(index).iter().rev() {
//                         iter_stack.push(Some(*child));
//                     }
//                 }
//             }
//             NodeValue::Range(node) => {
//                 let ranges = trie.get_range(&(node.start_index..node.end_index));
//             }
//         }

//         // Find children
//     }
// }
