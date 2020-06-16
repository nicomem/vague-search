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

// Don't directly implement AsBytes for all T because it could lead to errors.
// It is also marked as unsafe because
/// Return the raw byte representation of the value.
///
/// **Warning notes:**
/// - If the value contains a pointer or a reference,
/// the address will be present, not the pointed value.
/// - This representation is not portable.
pub fn as_bytes<T>(value: &T) -> &[u8] {
    unsafe { std::slice::from_raw_parts(value as *const T as *const u8, std::mem::size_of::<T>()) }
}
