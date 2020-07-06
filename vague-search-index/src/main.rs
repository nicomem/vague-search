//! The compiler binary of the vague-search project.
//!
//! Read a file composed of `<WORD> <FREQUENCY>` lines and create a compiled
//! dictionary from it.

use error::*;
use patricia_trie::PatriciaNode;
use snafu::*;
use std::{num::NonZeroU32, path::PathBuf};

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
    dbg!(args);

    let mut parent = PatriciaNode::create_empty();
    parent.insert(&String::from("abc"), NonZeroU32::new(1).unwrap());
    let child = parent.search(String::from("abc"));
    assert!(child.is_some());
    parent.delete(&String::from("abc"));

    let new_pat = PatriciaNode::create_from_file("words.txt");
    if let Ok(node) = new_pat {
        println!("Everything is ok!");
        println!("{:?}", node.search(String::from("ailley")));
    } else {
        println!("Ugh! Shit happened");
    }

    Ok(())
}
