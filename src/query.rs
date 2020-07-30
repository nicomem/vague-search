use crate::{
    error::*,
    layer_stack::LayerStack,
    search_approx::{search_approx, Distance, FoundWord, IterationStack, WordCharCount},
    search_exact::search_exact,
};
use snafu::*;
use std::{io::Write, num::NonZeroU32};
use vague_search_core::CompiledTrie;

/// Parse a command line and extract the query word and the searching distance.
fn parse_command_line(line: &str) -> Result<(&str, Distance)> {
    let mut split = line.split_whitespace();
    let action = split.next().context(CommandParse {
        line,
        cause: "No action found",
    })?;

    if action != "approx" {
        None.context(CommandParse {
            line,
            cause: "Action not recognized",
        })?
    }

    let dist = split
        .next()
        .context(CommandParse {
            line,
            cause: "No distance found",
        })?
        .parse()
        .ok()
        .context(CommandParse {
            line,
            cause: "Could not parse the distance into an integer",
        })?;

    let word = split.next().context(CommandParse {
        line,
        cause: "No word found",
    })?;

    Ok((word, dist))
}

/// Format the result (word, freq) to JSON and append it to the given buffer.
fn append_result_to_json(
    word: &str,
    freq: NonZeroU32,
    dist: Distance,
    mut json_writer: &mut impl Write,
) {
    // Add to the buffer: {"word":"<word>","freq":<freq>,"distance":<dist>}
    // Do not use format!() to avoid its overhead
    // Uses raw string literals: https://doc.rust-lang.org/reference/tokens.html#raw-string-literals
    let r = write!(&mut json_writer, r#"{{"word":"{}","freq":"#, word);
    debug_assert!(r.is_ok());

    let r = itoa::write(&mut json_writer, freq.get());
    debug_assert!(r.is_ok());

    let r = write!(&mut json_writer, r#","distance":"#);
    debug_assert!(r.is_ok());

    let r = itoa::write(&mut json_writer, dist);
    debug_assert!(r.is_ok());

    let r = write!(&mut json_writer, "}}");
    debug_assert!(r.is_ok());
}

/// Search for a word in the trie and return the result in a JSON representation.
fn process_search_exact(trie: &CompiledTrie, word: &str, json_writer: &mut impl Write) {
    // Search at a distance 0 and append the formatted result to the JSON buffer
    let r = write!(json_writer, "[");
    debug_assert!(r.is_ok());

    if let Some(freq) = search_exact(trie, word, None) {
        append_result_to_json(word, freq, 0, json_writer)
    }

    let r = writeln!(json_writer, "]");
    debug_assert!(r.is_ok());
}

/// Search for all words in the trie at a given distance (or less) of the query
/// and return the result in a JSON representation.
fn process_search_approx<'a>(
    trie: &'a CompiledTrie,
    word: &str,
    distance: Distance,
    layer_stack: &mut LayerStack<Distance, WordCharCount>,
    iter_stack: &mut IterationStack<'a>,
    result_buffer: &mut Vec<FoundWord>,
    json_writer: &mut impl Write,
) {
    // Clear the buffers of their old data
    layer_stack.clear();
    iter_stack.clear();
    result_buffer.clear();

    // Search at the query distance
    *result_buffer = search_approx(
        trie,
        word,
        distance,
        layer_stack,
        iter_stack,
        std::mem::take(result_buffer),
    );

    // Sort the results based on the order defined by FoundWord
    result_buffer.sort_unstable();

    let r = write!(json_writer, "[");
    debug_assert!(r.is_ok());

    let mut first = true;
    for found_word in result_buffer.iter_mut() {
        // Add comma between elements in the JSON array
        // But there must not be a trailing comma
        if !first {
            let r = write!(json_writer, ",");
            debug_assert!(r.is_ok());
        }
        first = true;

        // Extract inner string to reduce memory usage
        let inner_word = std::mem::take(&mut found_word.word);

        // Append the formatted result to the JSON buffer
        append_result_to_json(&inner_word, found_word.freq, found_word.dist, json_writer);
    }

    let r = writeln!(json_writer, "]");
    debug_assert!(r.is_ok());
}

/// Process queries received in the [standard input stream](std::io::stdin)
pub fn process_stdin_queries(trie: &CompiledTrie) -> Result<()> {
    const LINE_CAP: usize = 100;
    const LAYER_STACK_ELEMENTS_CAP: usize = 2000;
    const LAYER_STACK_LAYERS_CAP: usize = 50;
    const ITERATION_STACK_CAP: usize = 500;
    const RESULT_BUFFER_CAP: usize = 1000;

    // Initialize all buffers used to reduce allocation overhead
    let mut line = String::with_capacity(LINE_CAP);
    let mut layer_stack =
        LayerStack::with_capacity(LAYER_STACK_ELEMENTS_CAP, LAYER_STACK_LAYERS_CAP);
    let mut iter_stack = IterationStack::with_capacity(ITERATION_STACK_CAP);
    let mut result_buffer = Vec::with_capacity(RESULT_BUFFER_CAP);

    let input_stream = std::io::stdin();
    loop {
        line.clear();
        match input_stream.read_line(&mut line) {
            Ok(0) => return Ok(()), // EOF reached
            Ok(_) => {
                // Parse the command
                let (word, dist) = match parse_command_line(&line.trim()) {
                    Ok(e) => e,
                    Err(e) => {
                        eprintln!("> {}", e);
                        continue;
                    }
                };

                let stdout = std::io::stdout();
                let mut lock = stdout.lock();

                // Search and return the result in a JSON representation
                if dist == 0 {
                    process_search_exact(trie, word, &mut lock)
                } else {
                    process_search_approx(
                        trie,
                        word,
                        dist,
                        &mut layer_stack,
                        &mut iter_stack,
                        &mut result_buffer,
                        &mut lock,
                    )
                }
            }
            Err(e) => Err(e).context(Stdin)?,
        }
    }
}
