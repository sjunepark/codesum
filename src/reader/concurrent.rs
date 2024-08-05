use std::fmt::Debug;
use std::path::{Path, PathBuf};

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
    async fn aggregate<P>(&self, root: P) -> ReadResult
    where
        P: AsRef<Path> + Debug + Send + Sync + Clone + 'static,
    {
        let (files_to_read_sender_sync, files_to_read_receiver_sync) =
            crossbeam_channel::unbounded();
        let (files_to_read_sender, files_to_read_receiver) = async_channel::unbounded();
        let (strings_to_concat_sender, strings_to_concat_receiver) = async_channel::unbounded();

        let walk_handle = tokio::task::spawn_blocking(move || {
            debug!("Spawning walk task");
            walk(root.to_owned(), files_to_read_sender_sync)
        });

        let transfer_handle = tokio::spawn(async move {
            debug!("Spawning transfer_files task");
            while let Ok(path) = files_to_read_receiver_sync.recv() {
                files_to_read_sender
                    .send(path)
                    .await
                    .expect("Error sending file from sync to async channel");
            }
        });

        let read_handle = tokio::spawn({
            debug!("Spawning read_files task");
            read_files(files_to_read_receiver, strings_to_concat_sender)
        });

        let concat_handle = tokio::spawn({
            debug!("Spawning concat_strings task");
            concat_strings(strings_to_concat_receiver)
        });

        debug!("Waiting for tasks to finish");
        walk_handle.await.unwrap();
        transfer_handle.await.unwrap();
        read_handle.await.unwrap();
        concat_handle.await.unwrap()
    }
}

/// Recursively walks the directory tree starting from the given root path,
/// sending encountered files to the `files_to_read` channel.
///
/// This function is designed to be spawned as an independent task.
#[instrument(level = "debug", skip(files_to_read))]
fn walk<P>(root: P, files_to_read: crossbeam_channel::Sender<PathBuf>)
where
    P: AsRef<Path> + Debug + Clone,
{
    debug!("Start");
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
    debug!("Start")
}

#[instrument(level = "debug", skip(files_to_read, strings_to_concat))]
async fn read_files(
    files_to_read: async_channel::Receiver<PathBuf>,
    strings_to_concat: async_channel::Sender<String>,
) {
    debug!("Start");
    let mut tasks = Vec::new();

    while let Ok(path) = files_to_read.recv().await {
        let sender = strings_to_concat.clone();
        let task = tokio::spawn({
            debug!(?path, "Spawning read_file task");
            read_file(path, sender)
        });
        tasks.push(task);
    }

    debug!("Waiting for tasks to finish");
    for task in tasks.into_iter() {
        task.await.unwrap();
    }
    debug!("End");
}

#[instrument(level = "debug", skip(strings_to_concat))]
async fn read_file(path: PathBuf, strings_to_concat: async_channel::Sender<String>) {
    debug!("Start");
    let file_content = tokio::fs::read_to_string(&path).await;

    let absolute_path = path
        .canonicalize()
        .unwrap_or_else(|_| panic!("Error getting canonical path for {:?}", path));

    match file_content {
        Ok(file_content) => {
            strings_to_concat
                .send(file_content)
                .await
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
    debug!("End");
}

#[instrument(level = "debug", skip(strings_to_concat))]
async fn concat_strings(strings_to_concat: async_channel::Receiver<String>) -> ReadResult {
    debug!("Start");
    let mut content = String::new();
    let mut file_count = 0;

    while let Ok(file_content) = strings_to_concat.recv().await {
        trace!(
            file_content.len = file_content.len(),
            file_content,
            "Received file content"
        );
        content.push_str(&file_content);
        file_count += 1;
    }

    debug!("End");
    ReadResult {
        content,
        file_count,
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{ReaderTester, TestContext};

    use super::*;

    #[tokio::test]
    #[instrument]
    async fn test_read_files() {
        let _ = TestContext::new();
        trace!("Creating readers");
        let reader = AsyncReader::new();
        let content = reader.aggregate(".").await.content;

        let rt = ReaderTester::new();
        rt.test_for_current_crate(&content);
    }

    #[tokio::test]
    async fn hello() {
        println!("Hello");
    }
}
