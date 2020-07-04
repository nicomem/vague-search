use patricia_trie::PatriciaNode;
use std::num::NonZeroU32;

mod patricia_trie;
fn main() {
    println!("Hello, world!");
    let mut parent = PatriciaNode::create_empty();
    parent.insert(&String::from("abc"), NonZeroU32::new(1).unwrap());
    let child = parent.search(String::from("abc"));
    assert!(child.is_some());
    parent.delete(&String::from("abc"));
}
