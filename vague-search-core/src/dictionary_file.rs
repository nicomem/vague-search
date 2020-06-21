use crate::{
    error::*,
    trie::index::RangeElement,
    utils::{as_bytes, AsBytes},
    CompiledTrie, CompiledTrieNode,
};
use snafu::{ensure, ResultExt};
use std::{
    ffi::CStr,
    fs::{File, Metadata, OpenOptions},
    io::Write,
    mem::size_of,
    os::unix::io::IntoRawFd,
    path::Path,
};

/// The header of the dictionary file.
/// Contains information about the file structure, helping its parsing.
#[derive(Debug, Copy, Clone)]
pub struct Header {
    pub nb_nodes: usize,
    pub nb_chars: usize,
    pub nb_ranges: usize,
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
    mmap_ptr: *const libc::c_void,
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
    /// - `Vec<Node>`
    /// - `Vec<char>`
    /// - `Vec<RangeElement>`
    unsafe fn get_offsets_ptr(
        header: &Header,
        ptr: *const libc::c_void,
    ) -> (
        *const libc::c_void,
        *const libc::c_void,
        *const libc::c_void,
    ) {
        const HEADER_LEN: usize = size_of::<Header>();
        const NODE_LEN: usize = size_of::<CompiledTrieNode>();
        const CHAR_LEN: usize = size_of::<char>();

        let nodes_ptr = ptr.offset(HEADER_LEN as isize);
        let chars_ptr = nodes_ptr.offset((header.nb_nodes * NODE_LEN) as isize);
        let ranges_ptr = chars_ptr.offset((header.nb_chars * CHAR_LEN) as isize);

        (nodes_ptr, chars_ptr, ranges_ptr)
    }

    /// Try to read the dictionary from a file, previously written using the
    /// [write_file](DictionaryFile::write_file) method.
    /// Uses mmap internally to reduce memory usage.
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
        };

        // Return an error if mmap failed
        ensure!(
            mmap_ptr != libc::MAP_FAILED,
            FileMmap {
                path,
                strerror: unsafe { strerror() }.unwrap_or("Unknown")
            }
        );

        // Type and read the header
        let header = unsafe { *(mmap_ptr as *const Header) };

        // Type the compiled trie

        let trie = unsafe {
            // Get the offset pointers to each array
            let (nodes_ptr, chars_ptr, ranges_ptr) = Self::get_offsets_ptr(&header, mmap_ptr);

            // Type each array
            let nodes =
                std::slice::from_raw_parts(nodes_ptr as *const CompiledTrieNode, header.nb_nodes);
            let chars = std::slice::from_raw_parts(chars_ptr as *const char, header.nb_chars);
            let ranges =
                std::slice::from_raw_parts(ranges_ptr as *const RangeElement, header.nb_ranges);

            // Create a borrowing compiled trie
            CompiledTrie::from((nodes, chars, ranges))
        };

        Ok(Self {
            mmap_ptr,
            ptr_len: file_len,
            header,
            trie,
        })
    }

    /// Try to write the dictionary to a file.
    /// The file contents is not portable and must be read using the
    /// [read_file](DictionaryFile::read_file) method.
    pub fn write_file(&self, path: &Path) -> Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .context(FileOpen { path })?;

        // Write in the correct order:
        // - Header
        // - Nodes
        // - Chars
        // - Ranges
        let contents = [
            as_bytes(&self.header),
            self.trie.nodes().as_bytes(),
            self.trie.chars().as_bytes(),
            self.trie.ranges().as_bytes(),
        ];

        for bytes in &contents {
            file.write_all(bytes).context(FileWrite { path })?;
        }

        Ok(())
    }
}

impl Drop for DictionaryFile<'_> {
    fn drop(&mut self) {
        // munmap the inner pointer if the struct was read from a file
        if self.mmap_ptr != std::ptr::null() {
            unsafe { libc::munmap(self.mmap_ptr as *mut libc::c_void, self.ptr_len) };
        }
    }
}

impl<'a> From<CompiledTrie<'a>> for DictionaryFile<'a> {
    fn from(trie: CompiledTrie<'a>) -> Self {
        let header = Header {
            nb_nodes: trie.nodes().len(),
            nb_chars: trie.chars().len(),
            nb_ranges: trie.ranges().len(),
        };

        // Create a dictionary that is not mapped to a file
        Self {
            mmap_ptr: std::ptr::null(),
            ptr_len: 0,
            header,
            trie,
        }
    }
}
