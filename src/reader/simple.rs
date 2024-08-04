use std::fmt::Debug;
use std::path::Path;

use tracing::{error, instrument, trace, warn};

use crate::reader::Read;

#[derive(Debug)]
pub struct SimpleReader;

impl Read for SimpleReader {
    fn new() -> Self {
        SimpleReader
    }

    #[instrument(level = "trace")]
    fn read_files<P>(&self, root: P) -> String
    where
        P: AsRef<Path> + Debug,
    {
        let mut result = String::new();

        ignore::WalkBuilder::new(root)
            .build()
            .for_each(|entry| match entry {
                Ok(entry) => {
                    let path = entry.path();
                    let absolute_path = path.canonicalize().unwrap();
                    match entry.file_type() {
                        None => {
                            warn!(
                                ?absolute_path,
                                "Skipping file with file type stdin, which is not supported"
                            );
                        }
                        Some(file_type) if file_type.is_file() => {
                            // TODO: This is not the best implementation: It will read the whole file into memory – however large the file may be. Find a way to optimize it! (One idea might be to use a BufReader instead of read_to_string().)
                            let content = std::fs::read_to_string(path);
                            let content = content.unwrap_or_else(|error| {
                                error!(?absolute_path, ?error, "Error reading file");
                                String::new()
                            });
                            trace!(?absolute_path, "Read and concatenated file");
                            result.push_str(&content);
                        }
                        Some(_) => {
                            trace!(?path, extension = ?path.extension(), "Skipping non-file");
                        }
                    }
                }
                Err(error) => {
                    error!(%error, "Error reading file");
                }
            });

        result
    }
}

#[cfg(test)]
mod tests {
    use tracing::info;

    use crate::test_utils::TestContext;

    use super::*;

    #[test]
    fn test_read_files() {
        let tc = TestContext::new();

        let root = Path::new(".");
        let abs_root = root.canonicalize().unwrap();

        info!(root = ?abs_root, "Starting test_read_files");

        let reader = SimpleReader::new();
        let result = reader.read_files(root);

        let len = result.len();
        assert!(len > 0);

        tc.validate_result(&result);

        info!(?len, ?result, "Finished test_read_files");
    }
}
