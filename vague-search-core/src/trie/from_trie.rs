use crate::{trie::trie_node_interface::TrieNodeDrainer, *};
use std::{borrow::Cow, num::NonZeroU32, ops::Range};
use utils::char_dist;

#[derive(Debug, Eq, PartialEq)]
enum TrieNode<'a, N: TrieNodeDrainer> {
    Simple(&'a N, char),
    Patricia(&'a N, String),
    Range(&'a [N], Vec<char>),
}

/// Create a dummy index with an undefined (but fixed) value.
/// Useful when creating a temporary value, rewritten soon after.
const fn dummy_index() -> Option<IndexNodeNonZero> {
    // SAFETY: Safe because != 0
    // Use of unsafe because .unwrap() is not yet const fn
    Some(IndexNodeNonZero::new(unsafe {
        NonZeroU32::new_unchecked(u32::MAX)
    }))
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
    let end = IndexChar::new((pos + chars.len()) as u32);

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
    trie_ranges: &mut Vec<RangeElement>,
    nodes: &[N],
    range_chars: &[char],
) -> (Range<IndexRange>, char) {
    debug_assert_ne!(range_chars.len(), 0);
    debug_assert_eq!(nodes.len(), range_chars.len());

    let min = *range_chars.iter().min().unwrap();
    let max = *range_chars.iter().max().unwrap();
    // The range is inclusive (min and max in the range), so the length is max - min **+ 1**
    let range_len = max as usize - min as usize + 1;

    let index_range = IndexRange::new(trie_ranges.len() as u32)
        ..IndexRange::new((trie_ranges.len() + range_len) as u32);

    let char_to_index = |&c| c as usize - min as usize + *index_range.start as usize;

    // Create None values for the range
    trie_ranges.resize(
        trie_ranges.len() + range_len as usize,
        RangeElement {
            index_first_child: None,
            word_freq: None,
        },
    );

    for (i, node) in range_chars.iter().map(char_to_index).zip(nodes) {
        trie_ranges[i as usize] = RangeElement {
            // Use a dummy index to differentiate the element which are not in the trie
            // and the nodes which have a frequency of 0 (but have children which will
            // be inserted after).
            index_first_child: dummy_index(),
            word_freq: node.frequency(),
        }
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

/// Drain the characters of the nodes to then be used in [node_type_heuristic](node_type_heuristic).
/// We cannot call it there because it would make the return value have mutable reference to the nodes,
/// limiting nodes manipulation after.
fn extract_characters<N: TrieNodeDrainer>(nodes: &mut [N]) -> Vec<String> {
    nodes.iter_mut().map(|n| n.drain_characters()).collect()
}

/// Function designed to be used in [node_type_heuristic](node_type_heuristic).
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
        let first_char = chars.chars().next();

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

/// Create a node which has no information about its first child index.
fn create_partial_node<N: TrieNodeDrainer>(
    nb_siblings: u32,
    heuristic: TrieNode<N>,
    trie_chars: &mut String,
    trie_ranges: &mut Vec<RangeElement>,
) -> CompiledTrieNode {
    match heuristic {
        TrieNode::Simple(node, character) => CompiledTrieNode::NaiveNode(NaiveNode {
            nb_siblings,
            index_first_child: None,
            word_freq: node.frequency(),
            character,
        }),
        TrieNode::Patricia(node, node_chars) => {
            let char_range = add_chars(trie_chars, &node_chars);
            CompiledTrieNode::PatriciaNode(PatriciaNode {
                nb_siblings,
                index_first_child: None,
                word_freq: node.frequency(),
                char_range,
            })
        }
        TrieNode::Range(nodes, range_chars) => {
            let (range, first_char) = add_range(trie_ranges, nodes, &range_chars);
            CompiledTrieNode::RangeNode(RangeNode {
                nb_siblings,
                first_char,
                range,
            })
        }
    }
}

/// Append the information of the given node and its children
/// to the three [CompiledTrie](crate::CompiledTrie) vectors.
fn fill_from_trie<N: TrieNodeDrainer>(
    mut node: N,
    trie_nodes: &mut Vec<CompiledTrieNode>,
    trie_chars: &mut String,
    trie_ranges: &mut Vec<RangeElement>,
) {
    // Drain the children from the node and their characters
    let mut children = node.drain_children();
    let children_chars = extract_characters(&mut children);
    let heuristics = node_type_heuristic(&children, children_chars);
    let nb_created_nodes = heuristics.len();

    // Partially create the nodes in the heuristics.
    // Fill all information available without recursion.
    let partial_nodes =
        (0u32..nb_created_nodes as u32)
            .rev()
            .zip(heuristics)
            .map(|(nb_siblings, heuristic)| {
                create_partial_node(nb_siblings, heuristic, trie_chars, trie_ranges)
            });
    trie_nodes.extend(partial_nodes);

    // Call recursively and finish the partial nodes
    let mut partial_i = trie_nodes.len() - nb_created_nodes;
    let mut range_i = 0;
    for child in children.into_iter() {
        // The first child will be placed at the next index in the nodes vector
        let mut index_first_child = trie_nodes.len() as u32;

        // Call recursively with for the current node
        fill_from_trie(child, trie_nodes, trie_chars, trie_ranges);

        // If no new node added => no child
        // Set the index_first_child to 0 => Node field will be set to None because
        // it is a Option<NonZero>
        if trie_nodes.len() as u32 == index_first_child {
            index_first_child = 0;
        }

        // Finish the current partial node
        match trie_nodes[partial_i] {
            CompiledTrieNode::PatriciaNode(ref mut n) => {
                // Fill the partial node and advance to the next
                n.index_first_child = NonZeroU32::new(index_first_child).map(IndexNodeNonZero::new);
                partial_i += 1;
            }
            CompiledTrieNode::NaiveNode(ref mut n) => {
                // Fill the partial node and advance to the next
                n.index_first_child = NonZeroU32::new(index_first_child).map(IndexNodeNonZero::new);
                partial_i += 1;
            }
            CompiledTrieNode::RangeNode(ref n) => {
                // Get the index of the current element in the range
                let mut i = (*n.range.start + range_i) as usize;
                debug_assert!(i <= *n.range.end as usize);

                // Get the first element which is marked with the dummy index
                // (see the add_range function)
                // Since the range does not end with a not present element,
                // the loop does not out-of-bounds.
                while trie_ranges[i].index_first_child != dummy_index() {
                    i += 1;
                    debug_assert!(i <= *n.range.end as usize);
                }

                // Replace the dummy index with the correct one
                trie_ranges[i].index_first_child =
                    NonZeroU32::new(index_first_child).map(IndexNodeNonZero::new);

                // If we just filled the last element, advance to the next partial node
                // else advance in the range
                if i as u32 == *n.range.end - 1 {
                    partial_i += 1;
                    range_i = 0;
                } else {
                    range_i = i as u32 + 1 - *n.range.start;
                }
            }
        }
    }
}

impl<N: TrieNodeDrainer> From<N> for CompiledTrie<'_> {
    fn from(root: N) -> Self {
        const NODES_INIT_CAP: usize = 1024;
        const CHARS_INIT_CAP: usize = 512;
        const RANGES_INIT_CAP: usize = 256;

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
            let nb_chars_before = self.characters.len();

            let mut ret = String::new();
            std::mem::swap(&mut self.characters, &mut ret);

            if nb_chars_before == 0 {
                assert_eq!(ret, String::new());
            } else {
                assert_eq!(ret.len(), nb_chars_before);
            }
            assert_eq!(self.characters, String::new());
            ret
        }

        fn frequency(&self) -> Option<NonZeroU32> {
            self.frequency
        }

        fn drain_children(&mut self) -> Vec<Self> {
            let nb_children_before = self.children.len();

            let mut ret = Vec::new();
            std::mem::swap(&mut self.children, &mut ret);

            if nb_children_before == 0 {
                assert_eq!(ret, vec![]);
            } else {
                assert_eq!(ret.len(), nb_children_before);
            }
            assert_eq!(self.children, vec![]);
            ret
        }
    }

    fn create_simple(character: char, freq: u32, children: Vec<NodeDrainer>) -> NodeDrainer {
        NodeDrainer {
            characters: character.to_string(),
            frequency: NonZeroU32::new(freq),
            children,
        }
    }

    fn create_patricia(s: &str, freq: u32, children: Vec<NodeDrainer>) -> NodeDrainer {
        NodeDrainer {
            characters: s.to_string(),
            frequency: NonZeroU32::new(freq),
            children,
        }
    }

    fn run_assert_heuristic(
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
        let _ = run_assert_heuristic(&nodes, nodes_chars, vec![]);
    }

    #[test]
    fn test_heuristic_all_simple() {
        let mut nodes = vec![
            create_simple('a', 0, vec![]),
            create_simple('z', 0, vec![]),
            create_simple('🀄', 0, vec![]),
        ];
        let nodes_chars = extract_characters(&mut nodes);
        run_assert_heuristic(
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
            create_patricia("abaca", 0, vec![]),
            create_patricia("foobar", 0, vec![]),
            create_patricia(WEIRD_STRING, 0, vec![]),
        ];
        let nodes_chars = extract_characters(&mut nodes);
        run_assert_heuristic(
            &nodes,
            nodes_chars,
            vec![
                TrieNode::Patricia(&nodes[0], "abaca".to_string()),
                TrieNode::Patricia(&nodes[1], "foobar".to_string()),
                TrieNode::Patricia(&nodes[2], WEIRD_STRING.to_string()),
            ],
        );
    }

    fn create_range(range: Range<char>, step: usize) -> (Vec<char>, Vec<NodeDrainer>) {
        let chars: Vec<char> = ((range.start as u32)..(range.end as u32))
            .step_by(step)
            .flat_map(std::char::from_u32)
            .collect();

        let nodes = chars
            .iter()
            .map(|&c| c)
            .map(|c| create_simple(c, 0, vec![]))
            .collect();

        (chars, nodes)
    }

    #[test]
    fn test_heuristic_compact_ranges() {
        let (chars1, nodes1) = create_range('a'..'z', 1);
        let (chars2, nodes2) = create_range('←'..'⇿', 1);
        let (chars3, nodes3) = create_range('☀'..'⛿', 1);
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

        run_assert_heuristic(
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
        let (chars1, nodes1) = create_range('a'..'z', 1);
        let (chars2, nodes2) = create_range('←'..'⇿', 2);
        let (chars3, nodes3) = create_range('☀'..'⛿', 3);
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

        run_assert_heuristic(
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
        let (chars1, nodes1) = create_range('←'..'⇿', 2);

        const WEIRD_STRING: &str =
            "NΘ stop the an​*̶͑̾̾​̅ͫ͏̙̤g͇̫͛͆̾ͫ̑͆l͖͉̗̩̳̟̍ͫͥͨe̠̅s ͎a̧͈͖r̽̾̈́͒͑e n​ot rè̑ͧ̌aͨl̘̝̙̃ͤ͂̾̆ ZA̡͊͠͝LGΌ ISͮ̂҉̯͈͕̹̘̱ TO͇̹̺ͅƝ̴ȳ̳ TH̘Ë͖́̉ ͠P̯͍̭O̚​N̐Y̡ H̸̡̪̯ͨ͊̽̅̾̎Ȩ̬̩̾͛ͪ̈́̀́͘ ̶̧̨̱̹̭̯ͧ̾ͬC̷̙̲̝͖ͭ̏ͥͮ͟Oͮ͏̮̪̝͍M̲̖͊̒ͪͩͬ̚̚͜Ȇ̴̟̟͙̞ͩ͌͝S̨̥̫͎̭ͯ̿̔̀ͅ";

        let parts = vec![
            vec![
                create_patricia("abaca", 0, vec![]),
                create_simple('b', 0, vec![]),
                create_patricia("foobar", 0, vec![]),
            ],
            nodes1,
            vec![
                create_patricia(WEIRD_STRING, 0, vec![]),
                create_simple('🀄', 0, vec![]),
            ],
        ];
        let len1 = chars1.len();

        let mut nodes: Vec<_> = parts.into_iter().flatten().collect();
        let nodes_chars = extract_characters(&mut nodes);
        run_assert_heuristic(
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

    fn run_assert_from(
        root: NodeDrainer,
        target_nodes: &[CompiledTrieNode],
        target_chars: &str,
        target_ranges: &[RangeElement],
    ) {
        let compiled = CompiledTrie::from(root);
        assert_eq!(compiled.chars(), target_chars);
        assert_eq!(compiled.nodes(), target_nodes);
        assert_eq!(compiled.ranges(), target_ranges);
    }

    #[test]
    fn test_from_empty() {
        let root = create_simple('-', 0, vec![]);
        let target_nodes = vec![];
        let target_chars = "";
        let target_ranges = vec![];
        run_assert_from(root, &target_nodes, target_chars, &target_ranges);
    }

    #[test]
    fn test_from_all_naive() {
        let root = create_simple(
            '-',
            0,
            vec![
                create_simple(
                    'a',
                    1,
                    vec![create_simple('a', 2, vec![]), create_simple('h', 1, vec![])],
                ),
                create_simple('h', 0, vec![create_simple('a', 1, vec![])]),
                create_simple('z', 5, vec![]),
            ],
        );
        let target_nodes = vec![
            CompiledTrieNode::NaiveNode(NaiveNode {
                nb_siblings: 2,
                index_first_child: NonZeroU32::new(3).map(IndexNodeNonZero::new),
                word_freq: NonZeroU32::new(1),
                character: 'a',
            }),
            CompiledTrieNode::NaiveNode(NaiveNode {
                nb_siblings: 1,
                index_first_child: NonZeroU32::new(5).map(IndexNodeNonZero::new),
                word_freq: None,
                character: 'h',
            }),
            CompiledTrieNode::NaiveNode(NaiveNode {
                nb_siblings: 0,
                index_first_child: None,
                word_freq: NonZeroU32::new(5),
                character: 'z',
            }),
            CompiledTrieNode::NaiveNode(NaiveNode {
                nb_siblings: 1,
                index_first_child: None,
                word_freq: NonZeroU32::new(2),
                character: 'a',
            }),
            CompiledTrieNode::NaiveNode(NaiveNode {
                nb_siblings: 0,
                index_first_child: None,
                word_freq: NonZeroU32::new(1),
                character: 'h',
            }),
            CompiledTrieNode::NaiveNode(NaiveNode {
                nb_siblings: 0,
                index_first_child: None,
                word_freq: NonZeroU32::new(1),
                character: 'a',
            }),
        ];
        let target_chars = "";
        let target_ranges = vec![];
        run_assert_from(root, &target_nodes, target_chars, &target_ranges);
    }

    #[test]
    fn test_from_all_patricia() {
        let root = create_simple(
            '-',
            0,
            vec![
                create_patricia(
                    "aba",
                    1,
                    vec![
                        create_patricia("baba", 2, vec![]),
                        create_patricia("ca", 1, vec![]),
                    ],
                ),
                create_patricia("rota", 0, vec![create_patricia("ba", 1, vec![])]),
                create_patricia("super", 5, vec![]),
            ],
        );
        let target_nodes = vec![
            CompiledTrieNode::PatriciaNode(PatriciaNode {
                nb_siblings: 2,
                index_first_child: NonZeroU32::new(3).map(IndexNodeNonZero::new),
                word_freq: NonZeroU32::new(1),
                char_range: IndexChar::new(0)..IndexChar::new(3),
            }),
            CompiledTrieNode::PatriciaNode(PatriciaNode {
                nb_siblings: 1,
                index_first_child: NonZeroU32::new(5).map(IndexNodeNonZero::new),
                word_freq: None,
                char_range: IndexChar::new(3)..IndexChar::new(7),
            }),
            CompiledTrieNode::PatriciaNode(PatriciaNode {
                nb_siblings: 0,
                index_first_child: None,
                word_freq: NonZeroU32::new(5),
                char_range: IndexChar::new(7)..IndexChar::new(12),
            }),
            CompiledTrieNode::PatriciaNode(PatriciaNode {
                nb_siblings: 1,
                index_first_child: None,
                word_freq: NonZeroU32::new(2),
                char_range: IndexChar::new(12)..IndexChar::new(16),
            }),
            CompiledTrieNode::PatriciaNode(PatriciaNode {
                nb_siblings: 0,
                index_first_child: None,
                word_freq: NonZeroU32::new(1),
                char_range: IndexChar::new(16)..IndexChar::new(18),
            }),
            CompiledTrieNode::PatriciaNode(PatriciaNode {
                nb_siblings: 0,
                index_first_child: None,
                word_freq: NonZeroU32::new(1),
                // characters already present in the array, should reuse it
                char_range: IndexChar::new(1)..IndexChar::new(3),
            }),
        ];
        let target_chars = "abarotasuperbabaca";
        let target_ranges = vec![];
        run_assert_from(root, &target_nodes, target_chars, &target_ranges);
    }

    #[test]
    fn test_from_all_ranges() {
        let root = create_simple(
            '-',
            0,
            vec![
                create_simple(
                    'a',
                    1,
                    vec![create_simple('d', 2, vec![]), create_simple('f', 1, vec![])],
                ),
                create_simple(
                    'd',
                    0,
                    vec![
                        create_simple('a', 9, vec![]),
                        create_simple('c', 1, vec![]),
                        create_simple('r', 6, vec![]),
                        create_simple('t', 1, vec![]),
                        create_simple('w', 7, vec![]),
                    ],
                ),
                create_simple('f', 5, vec![]),
            ],
        );
        let target_nodes = vec![
            CompiledTrieNode::RangeNode(RangeNode {
                nb_siblings: 0,
                first_char: 'a',
                range: IndexRange::new(0)..IndexRange::new(6),
            }),
            CompiledTrieNode::RangeNode(RangeNode {
                nb_siblings: 0,
                first_char: 'd',
                range: IndexRange::new(6)..IndexRange::new(9),
            }),
            CompiledTrieNode::RangeNode(RangeNode {
                nb_siblings: 1,
                first_char: 'a',
                range: IndexRange::new(9)..IndexRange::new(12),
            }),
            CompiledTrieNode::RangeNode(RangeNode {
                nb_siblings: 0,
                first_char: 'r',
                range: IndexRange::new(12)..IndexRange::new(18),
            }),
        ];
        let target_chars = "";
        let target_ranges = vec![
            RangeElement {
                index_first_child: NonZeroU32::new(1).map(IndexNodeNonZero::new),
                word_freq: NonZeroU32::new(1),
            },
            RangeElement::default(),
            RangeElement::default(),
            RangeElement {
                index_first_child: NonZeroU32::new(2).map(IndexNodeNonZero::new),
                word_freq: None,
            },
            RangeElement::default(),
            RangeElement {
                index_first_child: None,
                word_freq: NonZeroU32::new(5),
            },
            RangeElement {
                index_first_child: None,
                word_freq: NonZeroU32::new(2),
            },
            RangeElement::default(),
            RangeElement {
                index_first_child: None,
                word_freq: NonZeroU32::new(1),
            },
            RangeElement {
                index_first_child: None,
                word_freq: NonZeroU32::new(9),
            },
            RangeElement::default(),
            RangeElement {
                index_first_child: None,
                word_freq: NonZeroU32::new(1),
            },
            RangeElement {
                index_first_child: None,
                word_freq: NonZeroU32::new(6),
            },
            RangeElement::default(),
            RangeElement {
                index_first_child: None,
                word_freq: NonZeroU32::new(1),
            },
            RangeElement::default(),
            RangeElement::default(),
            RangeElement {
                index_first_child: None,
                word_freq: NonZeroU32::new(7),
            },
        ];
        run_assert_from(root, &target_nodes, target_chars, &target_ranges);
    }

    #[test]
    fn test_from_mixed() {
        const HE_COMES: &str = "H̸̡̪̯ͨ͊̽̅̾̎Ȩ̬̩̾͛ͪ̈́̀́͘ ̶̧̨̱̹̭̯ͧ̾ͬC̷̙̲̝͖ͭ̏ͥͮ͟Oͮ͏̮̪̝͍M̲̖͊̒ͪͩͬ̚̚͜Ȇ̴̟̟͙̞ͩ͌͝S̨̥̫͎̭ͯ̿̔̀ͅ";
        const RUST_IS_LOVE: &str = "Rust is ❤";

        let root = create_simple(
            '-',
            0,
            vec![
                create_patricia(
                    "apata",
                    1,
                    vec![create_simple('d', 2, vec![]), create_simple('f', 1, vec![])],
                ),
                create_simple(
                    'd',
                    0,
                    vec![
                        create_simple('a', 9, vec![]),
                        create_patricia(HE_COMES, 1, vec![]),
                        create_simple('r', 6, vec![]),
                        create_simple('t', 1, vec![]),
                        create_simple('w', 7, vec![]),
                    ],
                ),
                create_simple('f', 5, vec![create_patricia(RUST_IS_LOVE, 999, vec![])]),
            ],
        );
        let target_nodes = vec![
            CompiledTrieNode::PatriciaNode(PatriciaNode {
                nb_siblings: 1,
                index_first_child: NonZeroU32::new(2).map(IndexNodeNonZero::new),
                word_freq: NonZeroU32::new(1),
                char_range: IndexChar::new(0)..IndexChar::new(5),
            }),
            CompiledTrieNode::RangeNode(RangeNode {
                nb_siblings: 0,
                first_char: 'd',
                range: IndexRange::new(0)..IndexRange::new(3),
            }),
            CompiledTrieNode::RangeNode(RangeNode {
                nb_siblings: 0,
                first_char: 'd',
                range: IndexRange::new(3)..IndexRange::new(6),
            }),
            CompiledTrieNode::NaiveNode(NaiveNode {
                nb_siblings: 2,
                index_first_child: None,
                word_freq: NonZeroU32::new(9),
                character: 'a',
            }),
            CompiledTrieNode::PatriciaNode(PatriciaNode {
                nb_siblings: 1,
                index_first_child: None,
                word_freq: NonZeroU32::new(1),
                char_range: IndexChar::new(5)..IndexChar::new(5 + HE_COMES.len() as u32),
            }),
            CompiledTrieNode::RangeNode(RangeNode {
                nb_siblings: 0,
                first_char: 'r',
                range: IndexRange::new(6)..IndexRange::new(12),
            }),
            CompiledTrieNode::PatriciaNode(PatriciaNode {
                nb_siblings: 0,
                index_first_child: None,
                word_freq: NonZeroU32::new(999),
                char_range: IndexChar::new(5 + HE_COMES.len() as u32)
                    ..IndexChar::new((5 + HE_COMES.len() + RUST_IS_LOVE.len()) as u32),
            }),
        ];
        let target_chars = ["apata", HE_COMES, RUST_IS_LOVE].concat();
        let target_ranges = vec![
            RangeElement {
                index_first_child: NonZeroU32::new(3).map(IndexNodeNonZero::new),
                word_freq: None,
            },
            RangeElement::default(),
            RangeElement {
                index_first_child: NonZeroU32::new(6).map(IndexNodeNonZero::new),
                word_freq: NonZeroU32::new(5),
            },
            RangeElement {
                index_first_child: None,
                word_freq: NonZeroU32::new(2),
            },
            RangeElement::default(),
            RangeElement {
                index_first_child: None,
                word_freq: NonZeroU32::new(1),
            },
            RangeElement {
                index_first_child: None,
                word_freq: NonZeroU32::new(6),
            },
            RangeElement::default(),
            RangeElement {
                index_first_child: None,
                word_freq: NonZeroU32::new(1),
            },
            RangeElement::default(),
            RangeElement::default(),
            RangeElement {
                index_first_child: None,
                word_freq: NonZeroU32::new(7),
            },
        ];
        run_assert_from(root, &target_nodes, &target_chars, &target_ranges);
    }
}
