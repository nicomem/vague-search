use snafu::Snafu;
use std::{
    fmt::{Debug, Display, Formatter},
    path::PathBuf,
};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Snafu)]
#[snafu(visibility(pub(crate)))] // Make creatable enum variants crate-visible
pub enum Error {
    #[snafu(display("Usage: {} /path/to/compiled/dict.bin", bin_name))]
    CliArgs { bin_name: String },
    #[snafu(display("Error while reading dictionary file {}: {}", path.display(), source))]
    DictionaryRead {
        path: PathBuf,
        source: vague_search_core::Error,
    },
}

// Link Error to Display to print the message when an error is returned from main.
// (taken from snafu issues, may be implemented in snafu in the future)
impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
