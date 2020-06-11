use crate::compiled_trie::*;
use crate::error::*;
use snafu::{ensure, ResultExt};
use std::os::unix::io::IntoRawFd;
use std::{
    ffi::CStr,
    fs::{File, Metadata},
    path::Path,
};

/// The header of the dictionary file.
/// Contains information about the file structure, helping its parsing.
#[derive(Debug, Copy, Clone)]
pub struct Header {
    pub nb_nodes: usize,
    pub nb_chars: usize,
}

/// The dictionary created by the index binary and saved in a file
/// to be later used by the search engine.
/// The same structure can be used for reading and writing.
///
/// When read, this structure holds the mmaped file and provides a safer
/// and easier interface to its content by typing the inner data,
/// without copying the entire file in memory.
#[derive(Debug)]
pub struct DictionaryFile<'a> {
    mmap_ptr: *const u8,
    ptr_len: usize,

    pub header: Header,
    pub trie: CompiledTrie<'a>,
}

/// Helper function to get the error string from errno after a failed libc function call.
unsafe fn strerror() -> Option<&'static str> {
    let errno = *libc::__errno_location();
    let strerror = libc::strerror(errno);
    let cstr = CStr::from_ptr(strerror);
    cstr.to_str().ok()
}

impl DictionaryFile<'_> {
    /// Return the offsets of the inner data which is composed of:
    /// - Vec<Node>
    /// - Vec<char>
    fn get_offsets(header: &Header) -> (isize, isize) {
        const HEADER_LEN: usize = std::mem::size_of::<Header>();
        const NODE_LEN: usize = std::mem::size_of::<CompiledTrieNode>();
        (
            HEADER_LEN as isize,
            (HEADER_LEN + header.nb_nodes * NODE_LEN) as isize,
        )
    }

    /// Read the structure from a file that have been written
    /// with the mmap_save method.
    /// In case of an error, return the corresponding POSIX error code.
    pub fn read_file(path: &Path) -> Result<Self> {
        // Open the file and read its length
        let file: File = File::open(path).context(FileOpen { path })?;
        let meta: Metadata = file.metadata().context(FileMeta { path })?;

        let fd = file.into_raw_fd();
        let file_len = meta.len() as usize;

        // mmap the file instead of reading it for speed and low memory consumption
        let mmap_ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                file_len,
                libc::PROT_READ,
                libc::MAP_SHARED,
                fd,
                0,
            )
        } as *const u8;

        // Return an error if mmap failed
        ensure!(
            mmap_ptr != libc::MAP_FAILED as *const u8,
            FileMmap {
                path,
                strerror: unsafe { strerror() }.unwrap_or("Unknown")
            }
        );

        // Type and read the header
        let header = unsafe { *(mmap_ptr as *const Header) };

        // Type the compiled trie
        let (nodes_offset, chars_offset) = Self::get_offsets(&header);
        let trie = unsafe {
            // Type the nodes vector
            let nodes_ptr = mmap_ptr.offset(nodes_offset);
            let nodes =
                std::slice::from_raw_parts(nodes_ptr as *const CompiledTrieNode, header.nb_nodes);

            // Type the chars vector
            let chars_ptr = mmap_ptr.offset(chars_offset);
            let chars = std::slice::from_raw_parts(chars_ptr as *const char, header.nb_chars);

            // Create a borrowing compiled trie
            (nodes, chars).into()
        };

        Ok(Self {
            mmap_ptr,
            ptr_len: file_len,
            header,
            trie,
        })
    }

    pub fn write_file(&self, path: &Path) -> Result<()> {
        todo!("Open file descriptor and mmap write it")
    }
}

impl Drop for DictionaryFile<'_> {
    fn drop(&mut self) {
        // munmap the inner pointer if the struct was read from a file
        if self.mmap_ptr != std::ptr::null() {
            unsafe { libc::munmap(self.mmap_ptr as *mut std::ffi::c_void, self.ptr_len) };
        }
    }
}

impl From<CompiledTrie<'_>> for DictionaryFile<'static> {
    fn from(trie: CompiledTrie<'_>) -> Self {
        todo!("Create DictionaryFile from CompiledTrie")
    }
}
