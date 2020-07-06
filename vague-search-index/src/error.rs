use snafu::Snafu;
use std::{num::ParseIntError, path::PathBuf};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))] // Make creatable enum variants crate-visible
pub enum Error {
    #[snafu(display("Could not open file {}: {}", path.display(), source))]
    FileOpen {
        path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("Could not read in file {}: {}", path.display(), source))]
    FileRead {
        path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("Could not divide in word and frequency in file {}: {}", path.display(), line))]
    ContentRead {
        path: PathBuf,
        line: String,
        number: usize,
    },
    #[snafu(display("Could not parse in non zero integer in file {}: {}", path.display(), source))]
    Parsing {
        path: PathBuf,
        source: ParseIntError,
    },
}
