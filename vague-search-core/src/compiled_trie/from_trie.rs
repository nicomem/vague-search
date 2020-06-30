use crate::{trie::trie_node_interface::TrieNodeDrainer, *};
use std::{borrow::Cow, ops::Range};
use utils::char_dist;

/// Add the characters to the vector and return its range of index.
/// If the characters are already present in the vector, it may not insert them
/// and instead return the already present characters range of index.
fn add_chars(big_string: &mut String, chars: &str) -> Range<IndexChar> {
    let pos = big_string.find(chars).unwrap_or_else(|| {
        // Save the start position where chars will be added
        let start_pos = big_string.len();
        big_string.push_str(chars);
        start_pos
    });

    let start = IndexChar::new(pos as u32);
    let end = IndexChar::new(big_string.len() as u32);

    start..end
}

enum TrieNode<'a, N: TrieNodeDrainer> {
    Simple(&'a N, char),
    Patricia(&'a N, String),
    Range(&'a [N], Vec<char>),
}

/// Check if the current character should be added to the current range.
fn should_add_to_range(range: &[char], cur: char) -> bool {
    // Because a RangeElement takes 4x less memory than a CompiledTrieNode,
    // we can allow 4 empty cells between 2 elements without taking more memory.
    // Moreover, since a range is faster than multiple nodes (indexing vs searching)
    // it is prefered in case they both take the same amount of memory.
    const MAX_DIST_IN_RANGE: i32 = 5;

    // Check the number of empty cells will be placed between the last character
    // in the range and the current if we add it.
    range
        .last()
        .map_or(false, |&last| char_dist(last, cur) <= MAX_DIST_IN_RANGE)
}

/// Find the best node types to create from the given nodes.
fn node_type_heuristic<'a, N: TrieNodeDrainer>(nodes: &'a mut [N]) -> Vec<TrieNode<'a, N>> {
    let mut res_nodes = Vec::new();
    let mut cur_range = Vec::new();

    // Drain the characters before looping to avoid the mutable reference to the nodes
    let node_chars: Vec<String> = nodes.iter_mut().map(|n| n.drain_characters()).collect();

    'for_nodes: for (i, (node, chars)) in nodes.iter().zip(node_chars).enumerate() {
        let is_one_char = chars.chars().nth(1).is_none();
        let first_char = chars.chars().nth(0);

        // Check the range state and either:
        // - add a character to the range and continue the loop
        // - extract the range as a SimpleNode
        // - extract the range as a RangeNode
        if is_one_char && should_add_to_range(&cur_range, first_char.unwrap()) {
            // Add the character to the range => RangeNode (not finished)
            cur_range.push(first_char.unwrap());

            // This node has been assigned, we can continue with the next
            continue 'for_nodes;
        } else if cur_range.len() == 1 {
            // There is only one character in the range => SimpleNode

            // Get the simple node, since there is only one character in the range,
            // the node is the one processed just before the current => i - 1
            let simple_node = &nodes[i - 1];
            res_nodes.push(TrieNode::Simple(simple_node, first_char.unwrap()));
        } else {
            // There is multiple characters in the range => RangeNode

            // Extract the range (and empty cur_range)
            let mut finished_range = Vec::new();
            cur_range.swap_with_slice(&mut finished_range);

            // Create the slice of the nodes in the range
            let range_len = finished_range.len();
            let slice = &nodes[(i - range_len)..i];

            // Push the finished range to the list of nodes to creates
            res_nodes.push(TrieNode::Range(slice, finished_range));
        }

        // Process the current node
        if is_one_char {
            // Begin the range => SimpleNode / RangeNode (not finished)
            cur_range.push(first_char.unwrap());
        } else {
            // Multiple characters => PatriciaNode
            res_nodes.push(TrieNode::Patricia(node, chars))
        }
    }

    res_nodes
}

/// Append the information of the given node and its children
/// to the three [CompiledTrie](crate::CompiledTrie) vectors.
fn fill_from_trie<N: TrieNodeDrainer>(
    mut node: N,
    nodes: &mut Vec<CompiledTrieNode>,
    big_string: &mut String,
    ranges: &mut Vec<RangeElement>,
) {
    todo!("Replace with the heuristic");

    // The start of the current layer, where children.len() elements
    // will be added just below
    let layer_start = nodes.len();
    let mut children = node.drain_children();
    let nb_children = children.len();

    // Fill the current node layer, without the index_first_child
    for (i, child) in children.iter_mut().enumerate() {
        let node_chars = child.drain_characters();
        let nb_siblings = (nb_children - i - 1) as u32;
        let word_freq = child.frequency();

        // Dummy value since only known after recursion
        let index_first_child = IndexNode::new(0);

        let node = if node_chars.len() == 1 {
            CompiledTrieNode::NaiveNode(NaiveNode {
                nb_siblings,
                index_first_child,
                word_freq,
                character: node_chars.chars().nth(0).unwrap(),
            })
        } else {
            let char_range = add_chars(big_string, &node_chars);
            CompiledTrieNode::PatriciaNode(PatriciaNode {
                nb_siblings,
                index_first_child,
                word_freq,
                char_range,
            })
        };

        // TODO: RangeNode

        nodes.push(node);
    }

    // Call recursively for the children
    for (i, child) in children.into_iter().enumerate() {
        // The first child will be placed at the next index in the nodes vector
        let index_first_child = nodes.len();

        // Call recursively with for the current node
        fill_from_trie(child, nodes, big_string, ranges);

        // Update the current node with the correct information
        match nodes[layer_start + i] {
            CompiledTrieNode::NaiveNode(ref mut n) => {
                n.index_first_child = IndexNode::new(index_first_child as u32)
            }
            CompiledTrieNode::PatriciaNode(ref mut n) => {
                n.index_first_child = IndexNode::new(index_first_child as u32)
            }
            CompiledTrieNode::RangeNode(_) => todo!("No range node currently"),
        }
    }
}

impl<N: TrieNodeDrainer> From<N> for CompiledTrie<'_> {
    fn from(root: N) -> Self {
        const NODES_INIT_CAP: usize = 256;
        const CHARS_INIT_CAP: usize = 256;
        const RANGES_INIT_CAP: usize = 0; // TODO: no ranges currently

        let mut nodes = Vec::with_capacity(NODES_INIT_CAP);
        let mut big_string = String::with_capacity(CHARS_INIT_CAP);
        let mut ranges = Vec::with_capacity(RANGES_INIT_CAP);

        fill_from_trie(root, &mut nodes, &mut big_string, &mut ranges);

        Self {
            nodes: Cow::Owned(nodes),
            chars: Cow::Owned(big_string),
            ranges: Cow::Owned(ranges),
        }
    }
}
