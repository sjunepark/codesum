use std::fmt::Debug;
use std::path::{Path, PathBuf};

use tokio::task::JoinHandle;
use tracing::{debug, error, instrument, trace, warn};

use crate::reader::Read;
use crate::ReadResult;

#[derive(Debug, Clone)]
pub struct AsyncReader {
    files_to_read: FilesToRead,
    strings_to_concat: StringsToConcat,
}

#[derive(Debug)]
struct AsyncWalker {
    files_to_read: FilesToRead,
}

impl AsyncWalker {
    fn new(files_to_read: FilesToRead) -> Self {
        AsyncWalker { files_to_read }
    }

    /// Recursively walks the directory tree starting from the given root path,
    /// sending encountered files to the `files_to_read` channel.
    ///
    /// This method is implemented to take ownership of `self` because it's designed
    /// to be spawned as an independent task.
    #[instrument(level = "trace", skip(self))]
    async fn walk<P>(self, root: P)
    where
        P: AsRef<Path> + Debug + Send + Sync + Clone + 'static,
    {
        debug!(?root, "Starting walk");
        ignore::WalkBuilder::new(root.clone())
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
                            trace!(?path, "Sending file to files_to_read channel");
                            self.files_to_read
                                .sender
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
}

#[derive(Debug, Clone)]
struct Channel<T> {
    sender: crossbeam_channel::Sender<T>,
    receiver: crossbeam_channel::Receiver<T>,
}

type FilesToRead = Channel<PathBuf>;
type StringsToConcat = Channel<String>;

impl Read for AsyncReader {
    fn new() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let files_to_read = FilesToRead { sender, receiver };

        let (sender, receiver) = crossbeam_channel::unbounded();
        let strings_to_concat = StringsToConcat { sender, receiver };

        AsyncReader {
            files_to_read,
            strings_to_concat,
        }
    }

    #[instrument(level = "trace", skip(self))]
    async fn read_files<P>(&self, root: P) -> ReadResult
    where
        P: AsRef<Path> + Debug + Send + Sync + Clone + 'static,
    {
        let files_to_read = self.files_to_read.clone();

        trace!("Spawning walk task");
        let walk_handle = tokio::spawn(AsyncWalker::new(files_to_read).walk(root.to_owned()));

        let receiver = self.files_to_read.receiver.clone();
        let sender = self.strings_to_concat.sender.clone();
        let read_handle = tokio::spawn(async move {
            let mut tasks = Vec::new();

            while let Ok(path) = receiver.recv() {
                trace!(?path, "Received file from files_to_read channel");
                // TODO: Is this multiple clone necessary? Is this a good approach?
                let sender = sender.clone();
                let task: JoinHandle<()> = tokio::spawn(async move {
                    // TODO: This is not logged, meaning that it's not being executed
                    trace!(?path, "Reading file content");
                    let file_content = tokio::fs::read_to_string(&path).await;
                    let file_content = file_content.unwrap_or_else(|error| {
                        error!(?path, ?error, "Error reading file");
                        String::new()
                    });
                    trace!(?path, "Sending file content to strings_to_concat channel");
                    sender.send(file_content).unwrap_or_else(|error| {
                        error!(
                            ?path,
                            ?error,
                            "Error sending file content to strings_to_concat channel"
                        );
                    });
                });
                tasks.push(task);
            }
            debug!("Finished reading files");
            tasks
        });

        let receiver = self.strings_to_concat.receiver.clone();
        let concat_handle = tokio::spawn(async move {
            let mut content = String::new();
            let mut file_count = 0;

            while let Ok(file_content) = receiver.recv() {
                content.push_str(&file_content);
                file_count += 1;
            }

            ReadResult {
                content,
                file_count,
            }
        });

        walk_handle.await.unwrap();
        let tasks = read_handle.await.unwrap();
        for task in tasks {
            task.await.unwrap();
        }
        concat_handle.await.unwrap()
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
