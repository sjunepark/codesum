use std::fmt::Debug;
use std::path::Path;

pub use concurrent::AsyncReader;
pub use simple::SimpleReader;

mod concurrent;
mod simple;

pub trait SyncRead {
    fn new() -> Self;

    /// Reads all files in the given path, and concatenates them into a single string.
    fn aggregate<P>(&self, root: P) -> ReadResult
    where
        P: AsRef<Path> + Debug;
}

pub trait Read {
    fn new() -> Self;

    fn aggregate<P>(&self, root: P) -> impl std::future::Future<Output = ReadResult> + Send
    where
        P: AsRef<Path> + Debug + Send + Sync + Clone + 'static;
}

pub struct ReadResult {
    pub content: String,
    pub file_count: usize,
}
