use std::{num::NonZeroUsize, ops::Range};

pub trait AsBytes {
    /// Return the raw byte representation of the value.
    ///
    /// **Warning notes:**
    /// - If the value contains a pointer or a reference,
    /// the address will be present, not the pointed value.
    /// - This representation is not portable.
    fn as_bytes(&self) -> &[u8];
}

// Implement AsBytes for slices
// It could still lead to some errors when T has a pointer (e.g. slice of vectors),
// but these case should not appear in this project.
impl<T> AsBytes for [T] {
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.as_ptr() as *const T as *const u8,
                self.len() * std::mem::size_of::<T>(),
            )
        }
    }
}

// Don't directly implement AsBytes for all T because it could lead to errors
// if implemented for any value.
/// Return the raw byte representation of the value.
///
/// **Warning notes:**
/// - If the value contains a pointer or a reference,
/// the address will be present, not the pointed value.
/// - This representation is not portable.
pub fn as_bytes<T>(value: &T) -> &[u8] {
    unsafe { std::slice::from_raw_parts(value as *const T as *const u8, std::mem::size_of::<T>()) }
}

/// Find the range of index of the query in the `last_n` elements in the slice.
/// If `last_n` is None, search in the entire slice.
pub fn find_subslice<T: Eq>(
    slice: &[T],
    query: &[T],
    last_n: Option<NonZeroUsize>,
) -> Option<Range<usize>> {
    let slice = if let Some(n) = last_n {
        &slice[(slice.len() - n.get())..]
    } else {
        slice
    };

    if slice.len() < query.len() {
        None
    } else {
        slice
            .windows(query.len())
            .position(|e| e == query)
            .map(|p| p..(p + query.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_subslice() {
        let vec = vec![1, 2, 3, 4];
        let query1 = vec![2, 3];
        let query2 = vec![1, 2, 4];
        let limit = NonZeroUsize::new(2).unwrap();

        assert_eq!(find_subslice(&vec, &query1, None), Some(1..3));
        assert_eq!(find_subslice(&vec, &query2, None), None);
        assert_eq!(find_subslice(&vec, &query1, Some(limit)), None);
    }
}
