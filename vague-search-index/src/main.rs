use patricia_trie::PatriciaNode;
use std::num::NonZeroU32;

mod patricia_trie;
mod utils;
fn main() {
    println!("Hello, world!");
    let mut parent = PatriciaNode::create_empty();
    parent.insert(&String::from("abc"), NonZeroU32::new(1).unwrap());
    let child = parent.search(String::from("abc"));
    assert!(child.is_some());
    parent.delete(&String::from("abc"));
    let new_pat = PatriciaNode::create_from_file("test");
    if let Some(node) = new_pat {
        println!("{:?}", node);
    }
}
