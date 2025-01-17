//! The compiler binary of the vague-search project.
//!
//! Read a file composed of `<WORD> <FREQUENCY>` lines and create a compiled
//! dictionary from it.

use error::*;
use patricia_trie::PatriciaNode;
use snafu::*;
use std::path::PathBuf;

use vague_search_core::{CompiledTrie, DictionaryFile};

mod error;
mod patricia_trie;
mod utils;

/// Represents the expected parsed program arguments.
#[derive(Debug)]
struct Args {
    words_path: PathBuf,
    dict_path: PathBuf,
}

/// Parse the arguments and return an error if the wrong number is given or a parsing error happens.
fn parse_args() -> Result<Args> {
    const BIN_NAME_DEFAULT: &str = "vague-search-index";
    let mut args = std::env::args();

    let bin_name = args.next().unwrap_or_else(|| BIN_NAME_DEFAULT.to_string());
    let cliargs_ctx = CliArgs {
        bin_name: &bin_name,
    };

    let words_path = args.next().context(cliargs_ctx)?.into();
    let dict_path = args.next().context(cliargs_ctx)?.into();

    // Make sure no more argument has been given
    if args.next().is_some() {
        None.context(cliargs_ctx)?;
    }

    Ok(Args {
        words_path,
        dict_path,
    })
}

fn main() -> Result<()> {
    let args = parse_args()?;

    eprintln!("Creating Patricia Trie from the file...");
    let mut patricia_trie = PatriciaNode::create_from_file(&args.words_path)?;

    eprintln!("Patricia Trie created, compressing...");
    patricia_trie.compress();
    let compiled: CompiledTrie = patricia_trie.into();
    let dict_file: DictionaryFile = compiled.into();

    eprintln!("Writing compressed dictionary...");
    dict_file.write_file(&args.dict_path).context(DictWrite {
        path: &args.dict_path,
    })
}
