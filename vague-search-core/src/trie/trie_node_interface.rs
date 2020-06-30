use std::num::NonZeroU32;

/// Methods to access and drain a trie.
/// Some methods will drain parts of the trie thus removing this part
/// from it to be used by the caller.
pub trait TrieNodeDrainer: Sized {
    /// Drain the characters associated to this node.
    fn drain_characters(&mut self) -> String;

    /// Return the frequency associated to an end node.
    /// If the node does not correspond to the end of a node, return None.
    fn frequency(&self) -> Option<NonZeroU32>;

    /// Drain the children of the node.
    fn drain_children(&mut self) -> Vec<Self>;
}
