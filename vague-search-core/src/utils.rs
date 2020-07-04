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

/// Compute the distance between two characters.
/// If a < b, the returned value will be positive.
/// If a > b, the returned value will be negative.
pub fn char_dist(a: char, b: char) -> i32 {
    b as i32 - a as i32
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_as_bytes_vec_i32() {
        let v = vec![i32::MIN, 42, i32::MAX];
        let bytes_custom = v.as_bytes();
        let bytes_manual: Vec<u8> =
            v.iter()
                .map(|n: &i32| n.to_ne_bytes())
                .fold(vec![], |mut acc, e| {
                    acc.extend_from_slice(&e);
                    acc
                });

        assert_eq!(bytes_custom, bytes_manual.as_slice());
    }

    #[test]
    fn test_as_bytes_i32() {
        let v = vec![i32::MIN, 42, i32::MAX];
        for n in v {
            assert_eq!(as_bytes(&n), n.to_ne_bytes());
        }
    }

    #[test]
    fn test_as_bytes_bytes() {
        let bytes = b"this is a test checking that trying to get a byte slice
        from a byte slice returns the same representation as before";

        let bytes2 = bytes.as_bytes();
        assert!(bytes.iter().zip(bytes2.iter()).all(|(a, b)| a == b));

        // We also check that we can apply the same function multiple times
        let bytes3 = bytes2.as_bytes();
        assert!(bytes.iter().zip(bytes3.iter()).all(|(a, b)| a == b));
    }

    #[test]
    fn test_char_dist() {
        assert_eq!(char_dist('f', 'f'), 0);
        assert_eq!(char_dist('a', 'z'), 25);
        assert_eq!(char_dist('z', 'a'), -25);
    }
}
