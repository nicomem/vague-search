use patricia_trie::PatriciaNode;
use std::num::NonZeroU32;

mod patricia_trie;
mod utils;
fn main() {
    let mut parent = PatriciaNode::create_empty();
    parent.insert(&String::from("abc"), NonZeroU32::new(1).unwrap());
    let child = parent.search(String::from("abc"));
    assert!(child.is_some());
    parent.delete(&String::from("abc"));
    
    let new_pat = PatriciaNode::create_from_file("words.txt");
    if let Some(node) = new_pat {
        println!("Everything is ok!");
        println!("{:?}", node.search(String::from("ailley")));
    }
    else {
        println!("Ugh! Shit happened");
    }
}
