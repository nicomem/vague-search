use std::num::NonZeroU32;

pub trait TrieNodeInterface: Sized {
    /// Give a hint about the number of nodes in the trie
    /// to optimize memory allocations.
    ///
    /// It **can be expected** that the implementer returns a number
    /// below or above the true number of nodes.
    ///
    /// It is **not expected** that the implementer returns a number
    /// that is much larger than the correct value.
    fn hint_nb_nodes(&self) -> usize;

    /// Return the characters associated to this node.
    fn characters(&self) -> &[char];

    /// Return the frequency associated to an end node.
    /// If the node does not correspond to the end of a node, return None.
    fn frequency(&self) -> Option<NonZeroU32>;

    /// Return an iterator over the children of the node.
    /// This iterator is sorted over the children character slices.
    fn children(&self) -> &[Self];
}
