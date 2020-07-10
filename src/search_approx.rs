// TODO Add search approx function

use crate::layer_stack::LayerStack;
use std::num::NonZeroU32;
use vague_search_core::CompiledTrie;

/// A type to store searching distances.
pub type Distance = u16;

/// A type to store word sizes.
pub type WordSize = u16;

/// A stack of iterations, used to linearise the recursive searching algorithm.
pub type IterationStack = Vec<()>; // TODO

/// A word that have been found by a search query.
pub struct FoundWord {
    pub word: String,
    pub freq: NonZeroU32,
    pub dist: Distance,
}

/// Search for all words in the trie at a given distance (or less) of the query.
///
/// Return a vector of all found words with their respective frequency.
pub fn search_approx(
    _trie: &CompiledTrie,
    _word: &str,
    _distance: Distance,
    _layer_stack: &mut LayerStack<Distance, WordSize>,
    _iter_stack: &mut IterationStack,
    _result_buffer: Vec<FoundWord>,
) -> Vec<FoundWord> {
    todo!()
}
