use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

/// Returns an Iterator to the Reader of the lines of the file
pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
