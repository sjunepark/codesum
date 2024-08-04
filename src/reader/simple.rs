use std::fmt::Debug;
use std::path::Path;

use tracing::{error, instrument, trace, warn};

use crate::reader::{Read, ReadResult};

#[derive(Debug)]
pub struct SimpleReader;

impl Read for SimpleReader {
    fn new() -> Self {
        SimpleReader
    }

    #[instrument(level = "trace")]
    fn read_files<P>(&self, root: P) -> ReadResult
    where
        P: AsRef<Path> + Debug,
    {
        let mut content = String::new();
        let mut file_count = 0;

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
                            let file_content = std::fs::read_to_string(path);
                            let file_content = file_content.unwrap_or_else(|error| {
                                error!(?absolute_path, ?error, "Error reading file");
                                String::new()
                            });
                            trace!(?absolute_path, "Read and concatenated file");
                            content.push_str(&file_content);
                            file_count += 1;
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

        ReadResult {
            content,
            file_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{ReaderTester, TestContext};

    use super::*;

    #[test]
    fn test_read_files() {
        let _ = TestContext::new();
        let reader = SimpleReader::new();
        let rt = ReaderTester::new(reader);

        rt.test_reader_for_current_crate();
    }
}
