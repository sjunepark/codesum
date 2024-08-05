use std::fmt::Debug;
use std::path::{Path, PathBuf};

use crossbeam_channel::{unbounded, Receiver, Sender};
use tokio::task::JoinHandle;
use tracing::{debug, error, instrument, trace, warn};

use crate::reader::Read;
use crate::ReadResult;

#[derive(Debug, Clone)]
pub struct AsyncReader;

impl Read for AsyncReader {
    fn new() -> Self {
        AsyncReader
    }

    #[instrument(level = "trace", skip(self))]
    async fn read_files<P>(&self, root: P) -> ReadResult
    where
        P: AsRef<Path> + Debug + Send + Sync + Clone + 'static,
    {
        let (files_to_read_sender, files_to_read_receiver) = unbounded();
        let (strings_to_concat_sender, strings_to_concat_receiver) = unbounded();

        trace!("Spawning walk task");
        let walk_handle = tokio::spawn(walk(root.to_owned(), files_to_read_sender));

        trace!("Spawning read task");
        let read_handle =
            tokio::spawn(read_files(files_to_read_receiver, strings_to_concat_sender));

        let concat_handle = tokio::spawn(concat_strings(strings_to_concat_receiver));

        walk_handle.await.unwrap();
        let tasks = read_handle.await.unwrap();
        for task in tasks {
            task.await.unwrap();
        }
        concat_handle.await.unwrap()
    }
}

/// Recursively walks the directory tree starting from the given root path,
/// sending encountered files to the `files_to_read` channel.
///
/// This function is designed to be spawned as an independent task.
#[instrument(level = "trace", skip(files_to_read))]
async fn walk<P>(root: P, files_to_read: Sender<PathBuf>)
where
    P: AsRef<Path> + Debug + Clone,
{
    debug!(?root, "Starting walk");
    ignore::WalkBuilder::new(root.clone())
        .build()
        .for_each(|entry| match entry {
            Ok(entry) => {
                let path = entry.path();
                let absolute_path = path.canonicalize().unwrap_or_else(|_| {
                    panic!("Error getting canonical path for {:?}", path);
                });
                match entry.file_type() {
                    None => {
                        warn!(
                            ?absolute_path,
                            "Skipping file with file type stdin, which is not supported"
                        );
                    }
                    Some(file_type) if file_type.is_file() => {
                        trace!(?path, "Sending file to files_to_read channel");
                        files_to_read
                            .send(path.to_path_buf())
                            .unwrap_or_else(|error| {
                                error!(
                                    ?absolute_path,
                                    ?error,
                                    "Error sending file to files_to_read channel"
                                );
                            });
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
    debug!(?root, "Finished walk");
}

#[instrument(level = "trace", skip(files_to_read, strings_to_concat))]
async fn read_files(
    files_to_read: Receiver<PathBuf>,
    strings_to_concat: Sender<String>,
) -> Vec<JoinHandle<()>> {
    let mut tasks = Vec::new();

    while let Ok(path) = files_to_read.recv() {
        let sender = strings_to_concat.clone();
        let task = tokio::spawn(read_file(path, sender));
        tasks.push(task);
    }
    tasks
}

#[instrument(level = "trace", skip(path, strings_to_concat))]
async fn read_file(path: PathBuf, strings_to_concat: Sender<String>) {
    let file_content = tokio::fs::read_to_string(&path).await;

    let absolute_path = path
        .canonicalize()
        .unwrap_or_else(|_| panic!("Error getting canonical path for {:?}", path));

    match file_content {
        Ok(file_content) => {
            strings_to_concat
                .send(file_content)
                .unwrap_or_else(|error| {
                    error!(
                        ?absolute_path,
                        ?error,
                        "Error sending file content to strings_to_concat channel"
                    );
                });
        }
        Err(error) => {
            error!(?absolute_path, ?error, "Error reading file");
        }
    }
}

#[instrument(level = "trace", skip(strings_to_concat))]
async fn concat_strings(strings_to_concat: Receiver<String>) -> ReadResult {
    let mut content = String::new();
    let mut file_count = 0;

    while let Ok(file_content) = strings_to_concat.recv() {
        content.push_str(&file_content);
        file_count += 1;
    }

    ReadResult {
        content,
        file_count,
    }
}

#[cfg(test)]
mod tests {
    use tracing::info;

    use crate::test_utils::TestContext;
    use crate::SimpleReader;
    use crate::SyncRead;

    use super::*;

    #[tokio::test]
    #[instrument]
    async fn test_read_files() {
        let _ = TestContext::new();
        trace!("Creating readers");
        let async_reader = AsyncReader::new();
        let sync_reader = SimpleReader::new();

        let sync_result = sync_reader.read_files(".").content;
        info!(?sync_result, "Sync result");
        let async_result = async_reader.read_files(".").await.content;

        assert_eq!(sync_result, async_result);
    }

    #[tokio::test]
    async fn hello() {
        println!("Hello");
    }
}
