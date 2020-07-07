//! Define index wrappers that can only be used to access their corresponding array.
//! If instead an index was returned as an integer, it could be used with any of
//! the array in the trie.
//! Also they can only be set inside this crate, so that the functions here
//! can be sure that these indices are valid (for the trie that provided them).

use std::{num::NonZeroU32, ops::Deref};

// Macro to implement slice indexing for corresponding index wrappers
macro_rules! index_wrappers {
    ($( $index:ident ),*) => {
        $(
            /// Represent a valid index in the [CompiledTrie](crate::CompiledTrie) corresponding array.
            /// **UB Warning:** This index must not be used on another trie than the one that provided it.
            #[derive(Debug, Copy, Clone, Eq, PartialEq)]
            pub struct $index {
                index: u32,
            }

            impl Deref for $index {
                type Target = u32;

                fn deref(&self) -> &Self::Target {
                    &self.index
                }
            }
        )*
    };
}

macro_rules! derive_new {
    ($ret:ident, $( $index:ident ),*) => {
        $(
            impl $index {
                pub(super) const fn new(index: $ret) -> Self {
                    Self { index }
                }
            }
        )*
    };
}

macro_rules! derive_from {
    ($index: ident, $( $into: ident ),+) => {
        $(
            impl From<$index> for $into {
                fn from(value: $index) -> Self {
                    u32::from(value.index) as $into
                }
            }
        )*
    };
}

/// Represent a valid index in the [CompiledTrie](crate::CompiledTrie) corresponding array.
/// Cannot represent the 0th index.
/// This enables some memory optimizations for [RangeElement](crate::RangeElement).
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct IndexNodeNonZero {
    index: NonZeroU32,
}

index_wrappers!(IndexChar, IndexRange);
derive_new!(u32, IndexChar, IndexRange);
derive_new!(NonZeroU32, IndexNodeNonZero);
derive_from!(IndexChar, u32, u64, usize);
derive_from!(IndexRange, u32, u64, usize);
derive_from!(IndexNodeNonZero, u32, u64, usize);
