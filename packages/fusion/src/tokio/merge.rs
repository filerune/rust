use std::path::{Path, PathBuf};

use tokio::{
    fs,
    io::{self, AsyncReadExt as _, AsyncWriteExt as _},
};

use crate::merge::{Merge, MergeError};

/// Trait for running the merge process.
pub trait MergeAsyncExt {
    /// Run the check process asynchronously.
    fn run_async(
        &self
    ) -> impl std::future::Future<Output = Result<bool, MergeError>> + Send;
}

impl MergeAsyncExt for Merge {
    async fn run_async(&self) -> Result<bool, MergeError> {
        let in_dir: &Path = match self.in_dir {
            | Some(ref p) => {
                let p: &Path = p.as_ref();

                // if in_dir not exists
                if !p.exists() {
                    return Err(MergeError::InDirNotFound);
                }

                // if in_dir not a directory
                if !p.is_dir() {
                    return Err(MergeError::InDirNotDir);
                }

                p
            },
            | None => return Err(MergeError::InDirNotSet),
        };

        let out_file: &Path = match self.out_file {
            | Some(ref p) => {
                let p: &Path = p.as_ref();

                // delete outpath target if exists
                if p.exists() {
                    if p.is_dir() {
                        fs::remove_dir_all(p)
                            .await
                            .map_err(|_| MergeError::OutFileNotRemoved)?;
                    } else {
                        fs::remove_file(p)
                            .await
                            .map_err(|_| MergeError::OutFileNotRemoved)?;
                    }
                }

                // create outpath
                if let Some(parent) = p.parent() {
                    fs::create_dir_all(parent)
                        .await
                        .map_err(|_| MergeError::OutDirNotCreated)?;
                }

                p
            },
            | None => return Err(MergeError::OutFileNotSet),
        };

        let buffer_capacity: usize = self.buffer_capacity;

        let output: fs::File = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(out_file)
            .await
            .map_err(|_| MergeError::OutFileNotOpened)?;

        // writer
        let mut writer: io::BufWriter<fs::File> =
            io::BufWriter::with_capacity(buffer_capacity, output);

        // get inputs
        let mut entries: Vec<PathBuf> = Vec::new();

        let mut read_dir: fs::ReadDir =
            fs::read_dir(in_dir).await.map_err(|_| MergeError::InDirNotRead)?;

        while let Some(ref entry) =
            read_dir.next_entry().await.map_err(|_| MergeError::InDirNotRead)?
        {
            let path: PathBuf = entry.path();

            if path.is_file() {
                entries.push(path);
            }
        }

        if entries.is_empty() {
            return Err(MergeError::InDirNoFile);
        }

        entries.sort_by_key(|entry| {
            entry
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .parse::<usize>()
                .unwrap()
        });

        // merge
        for entry in entries {
            let input: fs::File = fs::OpenOptions::new()
                .read(true)
                .open(&entry)
                .await
                .map_err(|_| MergeError::InFileNotOpened)?;

            let mut reader: io::BufReader<fs::File> =
                io::BufReader::with_capacity(buffer_capacity, input);

            let mut buffer: Vec<u8> = vec![0; buffer_capacity];

            loop {
                let read: usize = reader
                    .read(&mut buffer)
                    .await
                    .map_err(|_| MergeError::InFileNotRead)?;

                if read == 0 {
                    break;
                }

                writer
                    .write_all(&buffer[..read])
                    .await
                    .map_err(|_| MergeError::OutFileNotWritten)?;
            }
        }

        writer.flush().await.map_err(|_| MergeError::OutFileNotWritten)?;

        Ok(true)
    }
}
