use snafu::Snafu;
use std::fmt::{Debug, Display, Formatter};
use std::{num::ParseIntError, path::PathBuf};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Snafu)]
#[snafu(visibility(pub(crate)))] // Make creatable enum variants crate-visible
pub enum Error {
    #[snafu(display("Usage: {} /path/to/word/freq.txt /path/to/output/dict.bin", bin_name))]
    CliArgs { bin_name: String },
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
    #[snafu(display("Could not divide in word and frequency in file {} at line {}: {}", path.display(), number, line))]
    ContentRead {
        path: PathBuf,
        line: String,
        number: usize,
    },
    #[snafu(display("Could not parse in non zero integer in file {} at line {}: {}", path.display(), number, source))]
    Parsing {
        path: PathBuf,
        number: usize,
        source: ParseIntError,
    },
}

// Link Error to Display to print the message when an error is returned from main.
// (taken from snafu issues, may be implemented in snafu in the future)
impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
