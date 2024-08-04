use std::fmt::Debug;
use std::path::Path;

pub use simple::SimpleReader;

mod simple;

pub trait Read {
    fn new() -> Self;

    /// Reads all files in the given path, and concatenates them into a single string.
    fn read_files<P>(&self, root: P) -> String
    where
        P: AsRef<Path> + Debug;
}
