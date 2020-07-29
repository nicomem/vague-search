use crate::{layer_stack::LayerStack, search_exact::search_exact_children};
use std::{
    cmp::{min, Ordering},
    num::NonZeroU32,
};
use vague_search_core::{
    CompiledTrie, CompiledTrieNode, NaiveNode, NodeValue, PatriciaNode, RangeElement, RangeNode,
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
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FoundWord {
    fn cmp(&self, other: &Self) -> Ordering {
        self.dist
            .cmp(&other.dist)
            .then(other.freq.cmp(&self.freq))
            .then(self.word.cmp(&other.word))
    }
}

/// Retrieve and push the root nodes in the iteration stack.
/// Also push a dummy node (None) as the first element to indicate the end of the layer.
fn push_layer_nodes<'a>(
    iter_stack: &mut IterationStack<'a>,
    nodes: &'a [CompiledTrieNode],
    last_char: Option<char>,
) {
    iter_stack.reserve(nodes.len() + 1);

    // Push a dummy node to represent the end of the layer
    iter_stack.push(None);

    // Push the nodes in reverse order to pop them in the correct order in the future
    for node in nodes.iter().rev() {
        iter_stack.push(Some(IterationElement {
            node,
            last_char,
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
    last_char: Option<char>,
    cur_trie_char: char,
) {
    debug_assert_ne!(word, "");
    debug_assert_eq!(layer.len(), word.chars().count() + 1);
    debug_assert_eq!(last_layer.len(), layer.len());

    let mut word_chars = word.chars();
    let mut prev_word_char_opt = None;

    layer[0] = last_layer[0] + 1;
    for i in 1..layer.len() {
        // Retrieve the current character
        let cur_word_char = word_chars.next().unwrap();
        let diff_character = cur_word_char != cur_trie_char;

        // Compute the costs for insert/delete/replace
        let insert_cost = layer[i - 1] + 1;
        let delete_cost = last_layer[i] + 1;
        let replace_cost = last_layer[i - 1] + diff_character as Distance;

        // Compute transposition cost
        let trans_cost = match (
            parent_layer.is_empty(),
            prev_word_char_opt,
            cur_word_char,
            last_char,
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
            _ => Distance::MAX,
        };

        // Set the current cell value to the minimum of all costs
        layer[i] = min(min(insert_cost, delete_cost), min(replace_cost, trans_cost));

        // Save the current character for the next iteration
        prev_word_char_opt = Some(cur_word_char);
    }
}

/// Push the distance layers corresponding to the current [NaiveNode](NaiveNode).
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
    let [cur_layer, last_layer, parent_layer] = if layer_stack.nb_layers() >= 3 {
        unsafe { layer_stack.fetch_last_3_layers_unsafe() }
    } else {
        layer_stack.fetch_last_3_layers()
    };

    // Compute the distances and fill the layer with them
    compute_layer(
        cur_layer,
        last_layer,
        parent_layer,
        word,
        iter_elem.last_char,
        node.character,
    );
}

/// Push the distance layers corresponding to the current [PatriciaNode](PatriciaNode).
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
    let mut last_char = iter_elem.last_char;

    let mut has_at_least_3_layers = layer_stack.nb_layers() >= 3;

    // Do the same computation as a naive node for each character in the patricia node
    // It will create a new layer for each character, which is not the most performant but the easiest
    for ch in pat_chars.chars() {
        // Create a new empty layer
        layer_stack.push_layer(Some(ch), word_char_count + 1);

        // Get the last 3 layers needed for the distance computation
        let [cur_layer, last_layer, parent_layer] = if has_at_least_3_layers {
            unsafe { layer_stack.fetch_last_3_layers_unsafe() }
        } else {
            layer_stack.fetch_last_3_layers()
        };

        has_at_least_3_layers = true;

        // Compute the distances and fill the layer with them
        compute_layer(cur_layer, last_layer, parent_layer, word, last_char, ch);

        // Append a dummy node to indicate the end of the layer (character)
        push_layer_nodes(iter_stack, &[], None);

        // Modify the last char to the one which was just processed
        last_char = Some(ch);
    }

    let has_multiple_chars = pat_chars.chars().nth(1).is_some();
    if has_multiple_chars {
        // Remove the last dummy node since the last character is handled in the main parent loop
        let popped_node = iter_stack.pop();
        debug_assert!(matches!(popped_node, Some(None)));
    }
}

/// Find the index of the next range element
fn find_next_range_node(trie_ranges: &[RangeElement], current_range_index: usize) -> Option<usize> {
    // Find the position (after current index) of the first Some element
    let pos_opt = trie_ranges[current_range_index..]
        .iter()
        .position(|n| n.index_first_child.is_some() || n.word_freq.is_some())?;

    // Add the found position to the current index
    // Because the found position is based on the current index
    Some(current_range_index + pos_opt)
}

/// Check if the current index is the last of the range
fn is_last_index_of_range(index: u32, range_node: &RangeNode) -> bool {
    let range_len = u32::from(range_node.end_index) - u32::from(range_node.start_index);
    index + 1 >= range_len
}

/// Push the distance layers corresponding to the current [RangeNode](RangeNode).
fn push_layers_range<'a>(
    node: &RangeNode,
    iter_elem: &IterationElement<'a>,
    word: &str,
    word_char_count: WordCharCount,
    layer_stack: &mut LayerStack<Distance, WordCharCount>,
    iter_stack: &mut IterationStack<'a>,
    trie: &CompiledTrie,
) {
    // SAFETY: Safety checked during dictionary compilation
    let cur_trie_char =
        unsafe { std::char::from_u32_unchecked(node.first_char as u32 + iter_elem.range_offset) };

    // Create a new empty layer
    layer_stack.push_layer(Some(cur_trie_char), word_char_count + 1);

    // Get the last 3 layers needed for the distance computation
    let [cur_layer, last_layer, parent_layer] = if layer_stack.nb_layers() >= 3 {
        unsafe { layer_stack.fetch_last_3_layers_unsafe() }
    } else {
        layer_stack.fetch_last_3_layers()
    };

    // Compute the distances and fill the layer with them
    compute_layer(
        cur_layer,
        last_layer,
        parent_layer,
        word,
        iter_elem.last_char,
        cur_trie_char,
    );

    // Push the next range element if the current is not the last in the range
    if !is_last_index_of_range(iter_elem.range_offset, node) {
        // There remains some elements to do in the range
        // So the next one is pushed in the nodes stack
        let trie_ranges = trie.get_range(node.start_index, node.end_index);

        // The element at `iter_elem.range_offset` is the current one, so the search for the next element
        // needs to begin at the next one
        let next_possible_i = iter_elem.range_offset as usize + 1;
        let next_elem_i = find_next_range_node(trie_ranges, next_possible_i).unwrap();

        // Push the same node but with the incremented range offset
        // No dummy node is inserted because a sibling of the current node is inserted,
        // which is in the same layer
        iter_stack.push(Some(IterationElement {
            range_offset: next_elem_i as _,
            ..*iter_elem
        }));
    }
}

/// Process the current node and update the layer stack with the node's new layers.
fn push_layers_current_node<'a>(
    iter_elem: &IterationElement<'a>,
    word: &str,
    word_char_count: WordCharCount,
    trie: &CompiledTrie,
    layer_stack: &mut LayerStack<Distance, WordCharCount>,
    iter_stack: &mut IterationStack<'a>,
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
        NodeValue::Range(n) => push_layers_range(
            n,
            iter_elem,
            word,
            word_char_count,
            layer_stack,
            iter_stack,
            trie,
        ),
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
    *cur_layer
        .last()
        .unwrap_or_else(|| unsafe { std::hint::unreachable_unchecked() })
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

/// Compare the minimum distance in the layer with dist_max.
/// If there are equal elements, return their indices.
/// - ([5, 3, 2, 6], 3) -> (Less, [])
/// - ([5, 3, 4, 3], 3) -> (Equal, [1, 3])
/// - ([5, 6, 4, 4], 3) -> (Greater, [])
fn cmp_min_with_max_dist(cur_layer: &[Distance], dist_max: Distance) -> (Ordering, Vec<usize>) {
    let mut equals = Vec::new();

    // Compare each distance in the layer
    for (i, &e) in cur_layer.iter().enumerate() {
        match e.cmp(&dist_max) {
            // If one is less than dist_max, then the minimum must be less too
            Ordering::Less => return (Ordering::Less, Vec::new()),

            // If one is equal, the minimum could be less (but cannot be greater)
            // Add its index to the
            Ordering::Equal => equals.push(i),

            // If one is greater, no additional information can be deduced
            Ordering::Greater => {}
        }
    }

    // If we found an element equal (but no less), return this information
    // Else, all distances were greater
    let ord = if !equals.is_empty() {
        Ordering::Equal
    } else {
        Ordering::Greater
    };

    (ord, equals)
}

/// Get the children of the node. If the node does not have any, return an empty slice.
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
    index.map_or(Default::default(), |i| trie.get_siblings(i))
}

/// Get the last character of the current node.
fn get_current_last_char(trie: &CompiledTrie, iter_elem: &IterationElement) -> char {
    match iter_elem.node.node_value() {
        NodeValue::Naive(n) => n.character,
        NodeValue::Patricia(_) => {
            // SAFETY: Safe because in patricia node
            let pat_range = unsafe { iter_elem.node.patricia_range() };
            let substr = trie.get_chars(pat_range.start, pat_range.end);
            substr.chars().last().unwrap()
        }
        NodeValue::Range(n) => {
            // SAFETY: Safety checked during dictionary compilation
            unsafe { std::char::from_u32_unchecked(n.first_char as u32 + iter_elem.range_offset) }
        }
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
    push_layer_nodes(iter_stack, roots, None);
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

        if children.is_empty() {
            // If no children, remove its layer and continue with next iteration
            layer_stack.pop_layer();
        } else {
            // If children, compare the minimum distance of the layer with the max_dist
            match cmp_min_with_max_dist(cur_layer, dist_max) {
                // If it is less, add all children and continue with the next iteration
                (Ordering::Less, _) => {
                    // Get the last character of the current node
                    let last_char = get_current_last_char(trie, &iter_elem);

                    // Add all children to the stack and save the last char of their parent
                    push_layer_nodes(iter_stack, children, Some(last_char));
                }

                // If it is equal, it is now a problem of exact search, which can have
                // a better optimized algorithm than the approximate search
                (Ordering::Equal, equals) => {
                    // Search from all equal position
                    for equal_i in equals {
                        // The last index of the layer could be returned, which represent the end of the word
                        // This case is already handled in check_add_word_to_result
                        let split_index = if let Some((i, _)) = word.char_indices().nth(equal_i) {
                            i
                        } else {
                            continue;
                        };

                        // Find the portion of the word to search (remoPve the already searched part)
                        let subword_to_search = &word[split_index..];

                        // Search the subword from the children
                        let freq_opt = search_exact_children(trie, subword_to_search, children);
                        if let Some(freq) = freq_opt {
                            // Concatenate the already search subword with the newly searched subword
                            // to find the word that have been found
                            let mut word = layer_word.to_owned();
                            word.push_str(subword_to_search);

                            result_buffer.push(FoundWord {
                                word,
                                freq,
                                dist: dist_max,
                            })
                        }
                    }
                }

                // If it is greater, no children will have a result word,
                // so we can safely ignore them and pop the current layer
                (Ordering::Greater, _) => {
                    layer_stack.pop_layer();
                }
            }
        }
    }

    // Return the result buffer that has been filled in the stack loop
    result_buffer
}

#[cfg(test)]
mod test {
    use super::*;

    fn check_compute_layer_word(word: &str, trie_word: &str, target_layers: &[&[Distance]]) {
        let layer_len = word.chars().count() + 1;

        assert_eq!(target_layers.len(), trie_word.chars().count());

        let mut layer = vec![0; layer_len];
        let mut last_layer: Vec<_> = ((0 as Distance)..(layer_len as Distance)).collect();
        let mut parent_layer = vec![];
        let mut last_char = None;

        for (ch, target_layer) in trie_word.chars().zip(target_layers) {
            compute_layer(&mut layer, &last_layer, &parent_layer, word, last_char, ch);
            assert_eq!(&layer, target_layer);

            parent_layer = last_layer;
            last_layer = layer;
            layer = vec![0; layer_len];
            last_char = Some(ch);
        }
    }

    #[test]
    fn test_compute_layer_one_layer_same_char() {
        let word = "abaca";
        let trie_word = "a";
        let target_layers = [[1, 0, 1, 2, 3, 4].as_ref()];
        check_compute_layer_word(word, trie_word, &target_layers);
    }

    #[test]
    fn test_compute_layer_one_layer_same_not_first_char() {
        let word = "abaca";
        let trie_word = "c";
        let target_layers = [[1, 1, 2, 3, 3, 4].as_ref()];
        check_compute_layer_word(word, trie_word, &target_layers);
    }

    #[test]
    fn test_compute_layer_one_layer_same_diff_char() {
        let word = "abaca";
        let trie_word = "f";
        let target_layers = [[1, 1, 2, 3, 4, 5].as_ref()];
        check_compute_layer_word(word, trie_word, &target_layers);
    }

    #[test]
    fn test_compute_layer_kries_crise() {
        let word = "kries";
        let trie_word = "crise";
        let target_layers = [
            [1, 1, 2, 3, 4, 5].as_ref(),
            [2, 2, 1, 2, 3, 4].as_ref(),
            [3, 3, 2, 1, 2, 3].as_ref(),
            [4, 4, 3, 2, 2, 2].as_ref(),
            [5, 5, 4, 3, 2, 2].as_ref(),
        ];
        check_compute_layer_word(word, trie_word, &target_layers);
    }

    #[test]
    fn test_compute_layer_abaca_alabama() {
        let word = "abaca";
        let trie_word = "alabama";
        let target_layers = [
            [1, 0, 1, 2, 3, 4].as_ref(),
            [2, 1, 1, 2, 3, 4].as_ref(),
            [3, 2, 2, 1, 2, 3].as_ref(),
            [4, 3, 2, 2, 2, 3].as_ref(),
            [5, 4, 3, 2, 3, 2].as_ref(),
            [6, 5, 4, 3, 3, 3].as_ref(),
            [7, 6, 5, 4, 4, 3].as_ref(),
        ];
        check_compute_layer_word(word, trie_word, &target_layers);
    }

    #[test]
    fn test_compute_layer_alabama_abaca() {
        let word = "alabama";
        let trie_word = "abaca";
        let target_layers = [
            [1, 0, 1, 2, 3, 4, 5, 6].as_ref(),
            [2, 1, 1, 2, 2, 3, 4, 5].as_ref(),
            [3, 2, 2, 1, 2, 2, 3, 4].as_ref(),
            [4, 3, 3, 2, 2, 3, 3, 4].as_ref(),
            [5, 4, 4, 3, 3, 2, 3, 3].as_ref(),
        ];
        check_compute_layer_word(word, trie_word, &target_layers);
    }

    #[test]
    fn test_compute_layer_abcdef_badcfe() {
        let word = "abcdef";
        let trie_word = "badcfe";
        let target_layers = [
            [1, 1, 1, 2, 3, 4, 5].as_ref(),
            [2, 1, 1, 2, 3, 4, 5].as_ref(),
            [3, 2, 2, 2, 2, 3, 4].as_ref(),
            [4, 3, 3, 2, 2, 3, 4].as_ref(),
            [5, 4, 4, 3, 3, 3, 3].as_ref(),
            [6, 5, 5, 4, 4, 3, 3].as_ref(),
        ];
        check_compute_layer_word(word, trie_word, &target_layers);
    }

    #[test]
    fn test_cmp_min_with_max_dist_less() {
        let layer = vec![5, 3, 2, 6];
        let dist_max = 3;
        let (ord, v) = cmp_min_with_max_dist(&layer, dist_max);
        assert_eq!(ord, Ordering::Less);
        assert_eq!(v, Vec::new());
    }

    #[test]
    fn test_cmp_min_with_max_dist_one_equal() {
        let layer = vec![5, 3, 4, 6];
        let dist_max = 3;
        let (ord, v) = cmp_min_with_max_dist(&layer, dist_max);
        assert_eq!(ord, Ordering::Equal);
        assert_eq!(v, vec![1]);
    }

    #[test]
    fn test_cmp_min_with_max_dist_all_equal() {
        let layer = vec![3, 3, 3, 3];
        let dist_max = 3;
        let (ord, v) = cmp_min_with_max_dist(&layer, dist_max);
        assert_eq!(ord, Ordering::Equal);
        assert_eq!(v, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_cmp_min_with_max_dist_greater() {
        let layer = vec![5, 6, 4, 4];
        let dist_max = 3;
        let (ord, v) = cmp_min_with_max_dist(&layer, dist_max);
        assert_eq!(ord, Ordering::Greater);
        assert_eq!(v, Vec::new());
    }
}
