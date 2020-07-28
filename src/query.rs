use crate::{
    error::*,
    layer_stack::LayerStack,
    search_approx::{search_approx, Distance, FoundWord, IterationStack, WordCharCount},
    search_exact::search_exact,
};
use snafu::*;
use std::num::NonZeroU32;
use vague_search_core::CompiledTrie;

type Json = String;

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
fn append_result_to_json(word: &str, freq: NonZeroU32, dist: Distance, json_buffer: &mut Json) {
    // Add to the buffer: {"word":"<word>","freq":<freq>,"distance":<dist>}
    // Do not use format!() to avoid its overhead
    // Uses raw string literals: https://doc.rust-lang.org/reference/tokens.html#raw-string-literals
    json_buffer.push('{');
    json_buffer.push_str(r#""word":""#);
    json_buffer.push_str(word);
    json_buffer.push_str(r#"","freq":"#);
    json_buffer.push_str(&freq.to_string());
    json_buffer.push_str(r#","distance":"#);
    json_buffer.push_str(&dist.to_string());
    json_buffer.push('}');
}

/// Search for a word in the trie and return the result in a JSON representation.
fn process_search_exact(trie: &CompiledTrie, word: &str, mut json_buffer: Json) -> Json {
    // Clear the buffer of its old data
    json_buffer.clear();

    // Search at a distance 0 and append the formatted result to the JSON buffer
    json_buffer.push('[');
    if let Some(freq) = search_exact(trie, word, None) {
        append_result_to_json(word, freq, 0, &mut json_buffer)
    }
    json_buffer.push(']');

    json_buffer
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
    mut json_buffer: String,
) -> Json {
    // Clear the buffers of their old data
    layer_stack.clear();
    iter_stack.clear();
    result_buffer.clear();
    json_buffer.clear();

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

    json_buffer.push('[');
    for found_word in result_buffer.iter_mut() {
        // Extract inner string to reduce memory usage
        let inner_word = std::mem::take(&mut found_word.word);

        // Append the formatted result to the JSON buffer
        append_result_to_json(
            &inner_word,
            found_word.freq,
            found_word.dist,
            &mut json_buffer,
        );

        // Add comma between elements in the JSON array
        json_buffer.push(',');
    }
    // Remove the invalid trailing comma from the JSON array
    if !result_buffer.is_empty() {
        json_buffer.pop();
    }
    json_buffer.push(']');

    json_buffer
}

/// Display the JSON result in the [standard output stream](std::io::stdout)
fn display_json_result(json_buffer: &str) {
    println!("{}", json_buffer);
}

/// Process queries received in the [standard input stream](std::io::stdin)
pub fn process_stdin_queries(trie: &CompiledTrie) -> Result<()> {
    const LINE_CAP: usize = 30;
    const LAYER_STACK_ELEMENTS_CAP: usize = 100;
    const LAYER_STACK_LAYERS_CAP: usize = 10;
    const ITERATION_STACK_CAP: usize = 50;
    const RESULT_BUFFER_CAP: usize = 50;
    const JSON_BUFFER_CAP: usize = 300;

    // Initialize all buffers used to reduce allocation overhead
    let mut line = String::with_capacity(LINE_CAP);
    let mut json_buffer = Json::with_capacity(JSON_BUFFER_CAP);
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

                // Search and return the result in a JSON representation
                json_buffer = if dist == 0 {
                    process_search_exact(trie, word, json_buffer)
                } else {
                    process_search_approx(
                        trie,
                        word,
                        dist,
                        &mut layer_stack,
                        &mut iter_stack,
                        &mut result_buffer,
                        json_buffer,
                    )
                };

                display_json_result(&json_buffer);
            }
            Err(e) => Err(e).context(Stdin)?,
        }
    }
}
