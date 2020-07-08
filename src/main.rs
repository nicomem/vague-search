//! The application binary of the vague-search project.
//!
//! Listen for actions in [the standard input stream](std::io::stdin)
//! of the syntax `approx <N> <WORD>` to search for words in a
//! [distance](https://en.wikipedia.org/wiki/Damerau%E2%80%93Levenshtein_distance)
//! of at most N inside a compiled dictionary.
//!
//! See the [vague-search-index](../vague_search_index/index.html) crate for
//! documentation about the dictionary compiler binary.
//!
//! See the [vague-search-core](../vague_search_core/index.html) crate for
//! documentation about types and functions shared by the binaries.

use error::*;
use snafu::*;
use std::path::PathBuf;
use vague_search_core::DictionaryFile;
use levenshtein::distance_zero;

mod error;
mod levenshtein;

/// Represents the expected parsed program arguments.
#[derive(Debug)]
struct Args {
    dict_path: PathBuf,
}

/// Parse the arguments and return an error if the wrong number is given or a parsing error happens.
fn parse_args() -> Result<Args> {
    const BIN_NAME_DEFAULT: &str = "vague-search";
    let mut args = std::env::args();

    let bin_name = args.next().unwrap_or_else(|| BIN_NAME_DEFAULT.to_string());
    let cliargs_ctx = CliArgs {
        bin_name: &bin_name,
    };

    let dict_path = args.next().context(cliargs_ctx)?.into();

    // Make sure no more argument has been given
    if args.next().is_some() {
        None.context(cliargs_ctx)?;
    }

    Ok(Args { dict_path })
}

fn main() -> Result<()> {
    let args = parse_args()?;

    let dict_file = DictionaryFile::read_file(&args.dict_path).context(DictionaryRead {
        path: args.dict_path,
    })?;

    // TODO: Do the app
    dbg!(dict_file.trie.get_root_siblings().unwrap());

    dbg!(distance_zero(&dict_file.trie, "ala"));

    Ok(())
}
