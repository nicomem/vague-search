use std::{os::raw::c_int, path::Path};

#[derive(Debug)]
struct DataFile {}

impl DataFile {
    /// Read the structure from a file that have been written
    /// with the mmap_save method.
    /// In case of an error, return the corresponding POSIX error code.
    pub fn mmap_read(file: &Path) -> Result<Self, c_int> {
        todo!("Open file descriptor and mmap read it")
    }

    /// Write the structure to a file using the mmap libc syscall.
    /// In case of an error, return the corresponding POSIX error code.
    pub fn mmap_write(&self, file: &Path) -> Result<(), c_int> {
        todo!("Open file descriptor and mmap write it")
    }
}
