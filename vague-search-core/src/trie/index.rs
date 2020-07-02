//! Define index wrappers that can only be used to access their corresponding array.
//! If instead an index was returned as an integer, it could be used with any of
//! the array in the trie.
//! Here, we only implement indexing for the corresponding type of slice and the
//! inner index integer is kept private, to keep everything safe.

use std::{num::NonZeroU32, ops::Deref};

// Macro to implement slice indexing for corresponding index wrappers
macro_rules! index_wrappers {
    ($( $index:ident ),*) => {
        $(
            /// Represent a valid index in the [CompiledTrie](crate::CompiledTrie) corresponding array.
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

/// Same as [IndexNode](self::IndexNode) but cannot be 0.
/// This enables some memory optimizations for [RangeElement](self::RangeElement).
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

/// An element of the range array, accessible via a [RangeNode](crate::RangeNode).
/// Since `index_first_child` cannot have the value 0, the struct can be contained
/// inside an Option without using more memory.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RangeElement {
    /// The index of the first child in the node array.
    /// This index could not be equal to 0 because the 0th node is the trie root,
    /// which is a child to none.
    pub index_first_child: IndexNodeNonZero,

    /// The word frequency. If None, the word does not exist in the dictionary.
    pub word_freq: Option<NonZeroU32>,
}
