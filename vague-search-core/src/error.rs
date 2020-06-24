use snafu::Snafu;
use std::path::PathBuf;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))] // Make creatable enum variants crate-visible
pub enum Error {
    #[snafu(display("Could not open file {}: {}", path.display(), source))]
    FileOpen {
        path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("Could not get meta information about file {}: {}", path.display(), source))]
    FileMeta {
        path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("Could not mmap file {}: {}", path.display(), strerror))]
    FileMmap { path: PathBuf, strerror: String },
    #[snafu(display("Could not read in file {}: {}", path.display(), source))]
    FileRead {
        path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("Could not write in file {}: {}", path.display(), source))]
    FileWrite {
        path: PathBuf,
        source: std::io::Error,
    },
}
