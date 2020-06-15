pub trait AsBytes {
    /// Return the raw byte representation of the value.
    ///
    /// **Warning notes:**
    /// - If the value contains a pointer or a reference,
    /// the address will be present, not the pointed value.
    /// - This representation is not portable.
    unsafe fn as_bytes(&self) -> &[u8];
}

impl<T> AsBytes for [T] {
    unsafe fn as_bytes(&self) -> &[u8] {
        std::slice::from_raw_parts(
            self.as_ptr() as *const T as *const u8,
            self.len() * std::mem::size_of::<T>(),
        )
    }
}

impl<T> AsBytes for T {
    unsafe fn as_bytes(&self) -> &[u8] {
        std::slice::from_raw_parts((self as *const T) as *const u8, std::mem::size_of::<T>())
    }
}
