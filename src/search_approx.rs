use crate::layer_stack::LayerStack;
use std::{cmp::min, num::NonZeroU32};
use vague_search_core::{
    CompiledTrie, CompiledTrieNode, NaiveNode, NodeValue, PatriciaNode, RangeNode,
};

/// A type to store searching distances.
pub type Distance = u16;

/// A type to store word sizes.
pub type WordCharCount = u16;

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
/// Also push a dummy node (None) as the first element to indicate the end of the layer.
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
    layer_stack: &mut LayerStack<Distance, WordCharCount>,
    layer_char: Option<char>,
    char_count: WordCharCount,
) {
    // word_size + 1 because the first cell is for "no character" in the distance algorithm
    let layer = layer_stack.push_layer(layer_char, char_count + 1);
    for (i, e) in layer.iter_mut().enumerate() {
        *e = i as _;
    }
}

/// Fill the layer with the [Damerau-Levenshtein](https://en.wikipedia.org/wiki/Damerau%E2%80%93Levenshtein_distance)
/// distance computation.
fn compute_layer(
    layer: &mut [Distance],
    last_layer: &[Distance],
    parent_layer: &[Distance],
    word: &str,
    iter_elem: &IterationElement,
    cur_trie_char: char,
) {
    debug_assert_ne!(layer.len(), 0);
    debug_assert_eq!(last_layer.len(), layer.len());

    let mut word_chars = word.chars();
    let mut prev_word_char_opt = None;

    layer[0] = last_layer[0] + 1;
    for i in 1..layer.len() {
        // Retrieve the current character
        let cur_word_char = word_chars.next().unwrap();
        let same_character = cur_word_char == cur_trie_char;

        // Compute the costs for insert/delete/replace
        let insert_cost = layer[i - 1] + 1;
        let delete_cost = last_layer[i] + 1;
        let replace_cost = last_layer[i - 1] + same_character as Distance;

        // Compute transposition cost
        let trans_cost = match (
            parent_layer.is_empty(),
            prev_word_char_opt,
            cur_word_char,
            iter_elem.last_char,
            cur_trie_char,
        ) {
            // Check if transposing the 2 chars of one substring == the other
            (false, Some(word_prev), word_cur, Some(trie_prev), trie_cur)
                if word_prev == trie_cur && word_cur == trie_prev =>
            {
                debug_assert_eq!(parent_layer.len(), layer.len());
                parent_layer[i - 2] + 1
            }

            // If not, we cannot transpose and so return the max value to
            // make the min take one of the other costs
            _ => u16::MAX,
        };

        // Set the current cell value to the minimum of all costs
        layer[i] = min(min(insert_cost, delete_cost), min(replace_cost, trans_cost));

        // Save the current character for the next iteration
        prev_word_char_opt = Some(cur_word_char);
    }
}

fn push_layers_naive(
    node: &NaiveNode,
    iter_elem: &IterationElement,
    word: &str,
    word_char_count: WordCharCount,
    layer_stack: &mut LayerStack<Distance, WordCharCount>,
) {
    // Create a new empty layer
    layer_stack.push_layer(Some(node.character), word_char_count + 1);

    // Get the last 3 layers needed for the distance computation
    let [cur_layer, last_layer, parent_layer] = layer_stack.fetch_last_3_layers();

    // Compute the distances and fill the layer with them
    compute_layer(
        cur_layer,
        last_layer,
        parent_layer,
        word,
        iter_elem,
        node.character,
    );
}

fn push_layers_patricia(
    _node: &PatriciaNode,
    iter_elem: &IterationElement,
    word: &str,
    word_char_count: WordCharCount,
    layer_stack: &mut LayerStack<Distance, WordCharCount>,
    iter_stack: &mut IterationStack<'_>,
    trie: &CompiledTrie,
) {
    // Retrieve the patricia characters
    // SAFETY: Safe because in a patricia node
    let range_chars = unsafe { iter_elem.node.patricia_range() };
    let pat_chars = trie.get_chars(range_chars.start, range_chars.end);

    // Do the same computation as a naive node for each character in the patricia node
    // It will create a new layer for each character, which is not the most performant but the easiest
    for ch in pat_chars.chars() {
        // Create a new empty layer
        layer_stack.push_layer(Some(ch), word_char_count + 1);

        // Get the last 3 layers needed for the distance computation
        let [cur_layer, last_layer, parent_layer] = layer_stack.fetch_last_3_layers();

        // Compute the distances and fill the layer with them
        compute_layer(cur_layer, last_layer, parent_layer, word, iter_elem, ch);

        // Append a dummy node to indicate the end of the layer (character)
        push_layer_nodes(iter_stack, &[]);
    }

    // Remove the last dummy node since the last character is handled in the main parent loop
    let popped_node = iter_stack.pop();
    debug_assert!(matches!(popped_node, Some(None)));
}

fn push_layers_range(
    node: &RangeNode,
    iter_elem: &IterationElement,
    word: &str,
    word_char_count: WordCharCount,
    layer_stack: &mut LayerStack<Distance, WordCharCount>,
) {
    // SAFETY: Safety checked during dictionary compilation
    let cur_trie_char =
        unsafe { std::char::from_u32_unchecked(node.first_char as u32 + iter_elem.range_offset) };

    // Create a new empty layer
    layer_stack.push_layer(Some(cur_trie_char), word_char_count + 1);

    // Get the last 3 layers needed for the distance computation
    let [cur_layer, last_layer, parent_layer] = layer_stack.fetch_last_3_layers();

    // Compute the distances and fill the layer with them
    compute_layer(
        cur_layer,
        last_layer,
        parent_layer,
        word,
        iter_elem,
        cur_trie_char,
    );
}

/// Process the current node and update the layer stack with the node's new layers.
fn push_layers_current_node(
    iter_elem: &IterationElement,
    word: &str,
    word_char_count: WordCharCount,
    trie: &CompiledTrie,
    layer_stack: &mut LayerStack<Distance, WordCharCount>,
    iter_stack: &mut IterationStack<'_>,
) {
    match iter_elem.node.node_value() {
        NodeValue::Naive(n) => push_layers_naive(n, iter_elem, word, word_char_count, layer_stack),
        NodeValue::Patricia(n) => push_layers_patricia(
            n,
            iter_elem,
            word,
            word_char_count,
            layer_stack,
            iter_stack,
            trie,
        ),
        NodeValue::Range(n) => push_layers_range(n, iter_elem, word, word_char_count, layer_stack),
    }
}

/// Retrieve the node frequency.
fn get_node_frequency(iter_elem: &IterationElement, trie: &CompiledTrie) -> Option<NonZeroU32> {
    match iter_elem.node.node_value() {
        NodeValue::Naive(n) => n.word_freq,
        NodeValue::Patricia(n) => n.word_freq,
        NodeValue::Range(n) => {
            let slice = trie.get_range(n.start_index, n.end_index);
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
            let slice = trie.get_range(n.start_index, n.end_index);
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
    layer_stack: &mut LayerStack<Distance, WordCharCount>,
    iter_stack: &mut IterationStack<'a>,
    mut result_buffer: Vec<FoundWord>,
) -> Vec<FoundWord> {
    // Early return if nothing to search
    if word.is_empty() {
        return result_buffer;
    }

    // Retrieve the root nodes
    let roots = trie.get_root_siblings().unwrap();
    let word_char_count = word.chars().count();

    // Initialize both stacks
    push_layer_nodes(iter_stack, roots);
    push_first_layer(layer_stack, None, word_char_count as _);

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
        push_layers_current_node(
            &iter_elem,
            word,
            word_char_count as _,
            trie,
            layer_stack,
            iter_stack,
        );

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
