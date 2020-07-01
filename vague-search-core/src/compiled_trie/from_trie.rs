use crate::{trie::trie_node_interface::TrieNodeDrainer, *};
use std::{borrow::Cow, num::NonZeroU32, ops::Range};
use utils::char_dist;

#[derive(Debug, Eq, PartialEq)]
enum TrieNode<'a, N: TrieNodeDrainer> {
    Simple(&'a N, char),
    Patricia(&'a N, String),
    Range(&'a [N], Vec<char>),
}

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

/// Add a range to the trie ranges, and return its range of index and first character.
/// The created range is composed of:
/// - Partially completed values (for characters present in the slice)
/// - None values (for characters not present in the slice)
///
/// The partially created values is composed of as many information available in
/// the given parameters, the rest is filled with dummy values.
///
/// The fields filled with dummy values are:
/// - `index_first_child`
fn add_range<N: TrieNodeDrainer>(
    trie_ranges: &mut Vec<Option<RangeElement>>,
    nodes: &[N],
    range_chars: &[char],
) -> (Range<IndexRange>, char) {
    let dummy_index = IndexNodeNonZero::new(NonZeroU32::new(12345).unwrap());

    debug_assert_ne!(range_chars.len(), 0);
    debug_assert_eq!(nodes.len(), range_chars.len());

    let min = *range_chars.iter().min().unwrap();
    let max = *range_chars.iter().max().unwrap();
    let range_len = max as usize - min as usize;

    let index_range = IndexRange::new(trie_ranges.len() as u32)
        ..IndexRange::new((trie_ranges.len() + range_len) as u32);

    let char_to_index = |&c| c as usize - min as usize + *index_range.start as usize;

    // Create None values for the range
    trie_ranges.resize(trie_ranges.len() + range_len as usize, None);

    for (i, node) in range_chars.iter().map(char_to_index).zip(nodes) {
        trie_ranges[i as usize] = Some(RangeElement {
            index_first_child: dummy_index,
            word_freq: node.frequency(),
        })
    }

    (index_range, min)
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

/// Drain the characters of the nodes to then be used in [node_type_heuristic](self::node_type_heuristic).
/// We cannot call it there because it would make the return value have mutable reference to the nodes,
/// limiting nodes manipulation after.
fn extract_characters<N: TrieNodeDrainer>(nodes: &mut [N]) -> Vec<String> {
    nodes.iter_mut().map(|n| n.drain_characters()).collect()
}

/// Function designed to be used in [node_type_heuristic](self::node_type_heuristic).
/// Process the characters in the range to either:
/// - Do nothing (empty range)
/// - Add a SimpleNode to the `res_nodes` (single character in the range)
/// - Add a RangeNode to the `res_nodes` (multiple characters in the range)
fn process_range<'a, N: TrieNodeDrainer>(
    nodes: &'a [N],
    cur_range: &mut Vec<char>,
    res_nodes: &mut Vec<TrieNode<'a, N>>,
    index_cur_node: usize,
) {
    match cur_range.len() {
        0 => {}
        1 => {
            // There is only one character in the range => SimpleNode

            // Get the simple node, since there is only one character in the range,
            // the node is the one processed just before the current => index_cur_node - 1
            debug_assert_ne!(index_cur_node, 0);
            debug_assert!(index_cur_node <= nodes.len());
            let simple_node = &nodes[index_cur_node - 1];

            // Get the only character in the range and clear it
            let simple_char = cur_range[0];
            cur_range.clear();

            // Add the node to the result vector
            res_nodes.push(TrieNode::Simple(simple_node, simple_char));
        }
        _ => {
            // There is multiple characters in the range => RangeNode

            // Extract the range (and empty cur_range)
            let mut finished_range = Vec::new();
            std::mem::swap(cur_range, &mut finished_range);

            // Create the slice of the nodes in the range
            let range_len = finished_range.len();
            debug_assert!(index_cur_node <= nodes.len());
            debug_assert!(index_cur_node >= range_len);
            let slice = &nodes[(index_cur_node - range_len)..index_cur_node];

            // Push the finished range to the list of nodes to creates
            res_nodes.push(TrieNode::Range(slice, finished_range));
        }
    }
}

/// Find the best node types to create from the given nodes.
fn node_type_heuristic<N: TrieNodeDrainer>(
    nodes: &[N],
    nodes_chars: Vec<String>,
) -> Vec<TrieNode<'_, N>> {
    let mut res_nodes = Vec::new();
    let mut cur_range = Vec::new();

    'for_nodes: for (i, (node, chars)) in nodes.iter().zip(nodes_chars).enumerate() {
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
        } else {
            process_range(nodes, &mut cur_range, &mut res_nodes, i);
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

    process_range(nodes, &mut cur_range, &mut res_nodes, nodes.len());

    res_nodes
}

/// Append the information of the given node and its children
/// to the three [CompiledTrie](crate::CompiledTrie) vectors.
fn fill_from_trie<N: TrieNodeDrainer>(
    mut node: N,
    trie_nodes: &mut Vec<CompiledTrieNode>,
    trie_chars: &mut String,
    trie_ranges: &mut Vec<Option<RangeElement>>,
) {
    const DUMMY_INDEX: IndexNode = IndexNode::new(54321);

    // Drain the children from the node and their characters
    let mut children = node.drain_children();
    let children_chars = extract_characters(&mut children);
    let heuristics = node_type_heuristic(&children, children_chars);

    // Partially create the nodes in the heuristics.
    // Fill all information available without recursion.
    for (i, heuristic) in heuristics.iter().enumerate() {
        let nb_siblings = (heuristics.len() - i) as u32;
        let node = match heuristic {
            TrieNode::Simple(node, c) => CompiledTrieNode::NaiveNode(NaiveNode {
                nb_siblings,
                index_first_child: DUMMY_INDEX,
                word_freq: node.frequency(),
                character: *c,
            }),
            TrieNode::Patricia(node, node_chars) => {
                let char_range = add_chars(trie_chars, node_chars);
                CompiledTrieNode::PatriciaNode(PatriciaNode {
                    nb_siblings,
                    index_first_child: DUMMY_INDEX,
                    word_freq: node.frequency(),
                    char_range,
                })
            }
            TrieNode::Range(nodes, range_chars) => {
                let (range, first_char) = add_range(trie_ranges, nodes, range_chars);
                CompiledTrieNode::RangeNode(RangeNode {
                    nb_siblings,
                    first_char,
                    range,
                })
            }
        };

        trie_nodes.push(node);
    }
    todo!("Call recursively and fill the index_first_child");

    // // The start of the current layer, where children.len() elements
    // // will be added just below
    // let layer_start = trie_nodes.len();
    // let mut children = node.drain_children();
    // let nb_children = children.len();

    // // Fill the current node layer, without the index_first_child
    // for (i, child) in children.iter_mut().enumerate() {
    //     let node_chars = child.drain_characters();
    //     let nb_siblings = (nb_children - i - 1) as u32;
    //     let word_freq = child.frequency();

    //     // Dummy value since only known after recursion
    //     let index_first_child = IndexNode::new(0);

    //     let node = if node_chars.len() == 1 {
    //         CompiledTrieNode::NaiveNode(NaiveNode {
    //             nb_siblings,
    //             index_first_child,
    //             word_freq,
    //             character: node_chars.chars().nth(0).unwrap(),
    //         })
    //     } else {
    //         let char_range = add_chars(trie_chars, &node_chars);
    //         CompiledTrieNode::PatriciaNode(PatriciaNode {
    //             nb_siblings,
    //             index_first_child,
    //             word_freq,
    //             char_range,
    //         })
    //     };

    //     // TODO: RangeNode

    //     trie_nodes.push(node);
    // }

    // // Call recursively for the children
    // for (i, child) in children.into_iter().enumerate() {
    //     // The first child will be placed at the next index in the nodes vector
    //     let index_first_child = trie_nodes.len();

    //     // Call recursively with for the current node
    //     fill_from_trie(child, trie_nodes, trie_chars, trie_ranges);

    //     // Update the current node with the correct information
    //     match trie_nodes[layer_start + i] {
    //         CompiledTrieNode::NaiveNode(ref mut n) => {
    //             n.index_first_child = IndexNode::new(index_first_child as u32)
    //         }
    //         CompiledTrieNode::PatriciaNode(ref mut n) => {
    //             n.index_first_child = IndexNode::new(index_first_child as u32)
    //         }
    //         CompiledTrieNode::RangeNode(_) => todo!("No range node currently"),
    //     }
    // }
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

#[cfg(test)]
mod test {
    use super::*;
    use std::num::NonZeroU32;

    #[derive(Debug, Default, Clone, Eq, PartialEq)]
    struct NodeDrainer {
        pub characters: String,
        pub frequency: Option<NonZeroU32>,
        pub children: Vec<Self>,
    }

    impl TrieNodeDrainer for NodeDrainer {
        fn drain_characters(&mut self) -> String {
            let mut ret = String::new();
            std::mem::swap(&mut self.characters, &mut ret);
            assert_ne!(ret.len(), 0);
            assert_eq!(self.characters.len(), 0);
            ret
        }

        fn frequency(&self) -> Option<NonZeroU32> {
            self.frequency
        }

        fn drain_children(&mut self) -> Vec<Self> {
            let mut ret = Vec::new();
            self.children.swap_with_slice(&mut ret);
            assert_ne!(ret.len(), 0);
            assert_eq!(self.children.len(), 0);
            ret
        }
    }

    fn create_simple(character: char) -> NodeDrainer {
        NodeDrainer {
            characters: character.to_string(),
            frequency: None,
            children: vec![],
        }
    }

    fn create_patricia(s: &str) -> NodeDrainer {
        NodeDrainer {
            characters: s.to_string(),
            frequency: None,
            children: vec![],
        }
    }

    fn test_nodes(
        nodes: &Vec<NodeDrainer>,
        nodes_chars: Vec<String>,
        target: Vec<TrieNode<NodeDrainer>>,
    ) {
        let nb_nodes = nodes.len();
        let ret = node_type_heuristic(nodes, nodes_chars);
        assert_eq!(nodes.len(), nb_nodes);
        assert_eq!(ret, target);
    }

    #[test]
    fn test_heuristic_empty() {
        let mut nodes: Vec<NodeDrainer> = vec![];
        let nodes_chars = extract_characters(&mut nodes);
        let _ = test_nodes(&nodes, nodes_chars, vec![]);
    }

    #[test]
    fn test_heuristic_all_simple() {
        let mut nodes = vec![create_simple('a'), create_simple('z'), create_simple('🀄')];
        let nodes_chars = extract_characters(&mut nodes);
        test_nodes(
            &nodes,
            nodes_chars,
            vec![
                TrieNode::Simple(&nodes[0], 'a'),
                TrieNode::Simple(&nodes[1], 'z'),
                TrieNode::Simple(&nodes[2], '🀄'),
            ],
        );
    }

    #[test]
    fn test_heuristic_all_patricia() {
        // A weird unicode string, taken from the famously :
        // https://stackoverflow.com/questions/1732348/regex-match-open-tags-except-xhtml-self-contained-tags/1732454#1732454
        const WEIRD_STRING: &str =
            "NΘ stop the an​*̶͑̾̾​̅ͫ͏̙̤g͇̫͛͆̾ͫ̑͆l͖͉̗̩̳̟̍ͫͥͨe̠̅s ͎a̧͈͖r̽̾̈́͒͑e n​ot rè̑ͧ̌aͨl̘̝̙̃ͤ͂̾̆ ZA̡͊͠͝LGΌ ISͮ̂҉̯͈͕̹̘̱ TO͇̹̺ͅƝ̴ȳ̳ TH̘Ë͖́̉ ͠P̯͍̭O̚​N̐Y̡ H̸̡̪̯ͨ͊̽̅̾̎Ȩ̬̩̾͛ͪ̈́̀́͘ ̶̧̨̱̹̭̯ͧ̾ͬC̷̙̲̝͖ͭ̏ͥͮ͟Oͮ͏̮̪̝͍M̲̖͊̒ͪͩͬ̚̚͜Ȇ̴̟̟͙̞ͩ͌͝S̨̥̫͎̭ͯ̿̔̀ͅ";

        let mut nodes = vec![
            create_patricia("abaca"),
            create_patricia("foobar"),
            create_patricia(WEIRD_STRING),
        ];
        let nodes_chars = extract_characters(&mut nodes);
        test_nodes(
            &nodes,
            nodes_chars,
            vec![
                TrieNode::Patricia(&nodes[0], "abaca".to_string()),
                TrieNode::Patricia(&nodes[1], "foobar".to_string()),
                TrieNode::Patricia(&nodes[2], WEIRD_STRING.to_string()),
            ],
        );
    }

    fn create_range_chars(range: Range<char>, step: usize) -> Vec<char> {
        ((range.start as u32)..(range.end as u32))
            .step_by(step)
            .flat_map(std::char::from_u32)
            .collect()
    }

    #[test]
    fn test_heuristic_compact_ranges() {
        let chars1 = create_range_chars('a'..'z', 1);
        let nodes1: Vec<_> = chars1.iter().map(|&c| c).map(create_simple).collect();
        let chars2 = create_range_chars('←'..'⇿', 1);
        let nodes2: Vec<_> = chars2.iter().map(|&c| c).map(create_simple).collect();
        let chars3 = create_range_chars('☀'..'⛿', 1);
        let nodes3: Vec<_> = chars3.iter().map(|&c| c).map(create_simple).collect();
        let mut nodes = nodes1
            .into_iter()
            .chain(nodes2)
            .chain(nodes3)
            .collect::<Vec<_>>();

        let nodes_chars = extract_characters(&mut nodes);
        let (len1, len2) = (chars1.len(), chars2.len());

        assert_ne!(chars1.len(), 0);
        assert_ne!(chars2.len(), 0);
        assert_ne!(chars3.len(), 0);
        assert_eq!(nodes.len(), chars1.len() + chars2.len() + chars3.len());

        test_nodes(
            &nodes,
            nodes_chars,
            vec![
                TrieNode::Range(&nodes[..len1], chars1),
                TrieNode::Range(&nodes[len1..(len1 + len2)], chars2),
                TrieNode::Range(&nodes[(len1 + len2)..], chars3),
            ],
        );
    }

    #[test]
    fn test_heuristic_partial_ranges() {
        let chars1 = create_range_chars('a'..'z', 1);
        let nodes1: Vec<_> = chars1.iter().map(|&c| c).map(create_simple).collect();
        let chars2 = create_range_chars('←'..'⇿', 2);
        let nodes2: Vec<_> = chars2.iter().map(|&c| c).map(create_simple).collect();
        let chars3 = create_range_chars('☀'..'⛿', 3);
        let nodes3: Vec<_> = chars3.iter().map(|&c| c).map(create_simple).collect();
        let mut nodes = nodes1
            .into_iter()
            .chain(nodes2)
            .chain(nodes3)
            .collect::<Vec<_>>();

        let nodes_chars = extract_characters(&mut nodes);
        let (len1, len2) = (chars1.len(), chars2.len());

        assert_ne!(chars1.len(), 0);
        assert_ne!(chars2.len(), 0);
        assert_ne!(chars3.len(), 0);
        assert_eq!(nodes.len(), chars1.len() + chars2.len() + chars3.len());

        test_nodes(
            &nodes,
            nodes_chars,
            vec![
                TrieNode::Range(&nodes[..len1], chars1),
                TrieNode::Range(&nodes[len1..(len1 + len2)], chars2),
                TrieNode::Range(&nodes[(len1 + len2)..], chars3),
            ],
        );
    }

    #[test]
    fn test_heuristic_mixed() {
        let chars1 = create_range_chars('←'..'⇿', 2);
        let range1: Vec<_> = chars1.iter().map(|&c| c).map(create_simple).collect();

        const WEIRD_STRING: &str =
            "NΘ stop the an​*̶͑̾̾​̅ͫ͏̙̤g͇̫͛͆̾ͫ̑͆l͖͉̗̩̳̟̍ͫͥͨe̠̅s ͎a̧͈͖r̽̾̈́͒͑e n​ot rè̑ͧ̌aͨl̘̝̙̃ͤ͂̾̆ ZA̡͊͠͝LGΌ ISͮ̂҉̯͈͕̹̘̱ TO͇̹̺ͅƝ̴ȳ̳ TH̘Ë͖́̉ ͠P̯͍̭O̚​N̐Y̡ H̸̡̪̯ͨ͊̽̅̾̎Ȩ̬̩̾͛ͪ̈́̀́͘ ̶̧̨̱̹̭̯ͧ̾ͬC̷̙̲̝͖ͭ̏ͥͮ͟Oͮ͏̮̪̝͍M̲̖͊̒ͪͩͬ̚̚͜Ȇ̴̟̟͙̞ͩ͌͝S̨̥̫͎̭ͯ̿̔̀ͅ";

        let parts = vec![
            vec![
                create_patricia("abaca"),
                create_simple('b'),
                create_patricia("foobar"),
            ],
            range1,
            vec![create_patricia(WEIRD_STRING), create_simple('🀄')],
        ];
        let len1 = chars1.len();

        let mut nodes: Vec<_> = parts.into_iter().flatten().collect();
        let nodes_chars = extract_characters(&mut nodes);
        test_nodes(
            &nodes,
            nodes_chars,
            vec![
                TrieNode::Patricia(&nodes[0], "abaca".to_string()),
                TrieNode::Simple(&nodes[1], 'b'),
                TrieNode::Patricia(&nodes[2], "foobar".to_string()),
                TrieNode::Range(&nodes[3..(3 + len1)], chars1),
                TrieNode::Patricia(&nodes[3 + len1], WEIRD_STRING.to_string()),
                TrieNode::Simple(&nodes[4 + len1], '🀄'),
            ],
        );
    }
}
