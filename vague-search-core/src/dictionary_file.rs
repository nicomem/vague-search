use crate::{compiled_trie::*, error::*, utils::AsBytes};
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
    /// - Vec<Node>
    /// - Vec<char>
    fn get_offsets(header: &Header) -> (isize, isize) {
        const HEADER_LEN: usize = size_of::<Header>();
        const NODE_LEN: usize = size_of::<CompiledTrieNode>();
        (
            HEADER_LEN as isize,
            (HEADER_LEN + header.nb_nodes * NODE_LEN) as isize,
        )
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
        let contents = [
            unsafe { self.header.as_bytes() },
            self.trie.nodes_bytes(),
            self.trie.chars_bytes(),
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

impl From<CompiledTrie<'_>> for DictionaryFile<'static> {
    fn from(_trie: CompiledTrie<'_>) -> Self {
        todo!("Create DictionaryFile from CompiledTrie")
    }
}
