use std::{
    fs::Metadata,
    path::{Path, PathBuf},
};

use smol::fs;

use crate::check::{Check, CheckError, MissingChunks, SizeMismatch};

/// Trait for running the check process.
pub trait CheckAsyncExt {
    /// Run the check process asynchronously.
    fn run_async(
        &self
    ) -> impl std::future::Future<Output = Result<(), CheckError>> + Send;
}

impl CheckAsyncExt for Check {
    async fn run_async(&self) -> Result<(), CheckError> {
        let in_dir: &Path = match self.in_dir {
            | Some(ref p) => {
                let p: &Path = p.as_ref();

                // if in_dir not exists
                if !p.exists() {
                    return Err(CheckError::InDirNotFound);
                }

                // if in_dir not a directory
                if !p.is_dir() {
                    return Err(CheckError::InDirNotDir);
                }

                p
            },
            | None => return Err(CheckError::InDirNotSet),
        };

        let file_size: usize =
            self.file_size.ok_or(CheckError::FileSizeNotSet)?;

        let total_chunks: usize =
            self.total_chunks.ok_or(CheckError::TotalChunksNotSet)?;

        let mut actual_size: usize = 0;
        let mut missing: Vec<usize> = Vec::with_capacity(total_chunks);

        for i in 0..total_chunks {
            let target_file: PathBuf = in_dir.join(i.to_string());

            let file: fs::File = match fs::OpenOptions::new()
                .read(true)
                .open(&target_file)
                .await
            {
                | Ok(f) => f,
                | Err(_) => {
                    missing.push(i);
                    continue;
                },
            };

            let metadata: Metadata =
                file.metadata().await.map_err(|_| CheckError::InFileNotRead)?;

            if !metadata.is_file() {
                missing.push(i);
                continue;
            }

            actual_size += metadata.len() as usize;
        }

        if !missing.is_empty() {
            return Err(CheckError::MissingChunks(MissingChunks { missing }));
        }

        if file_size != actual_size {
            return Err(CheckError::SizeMismatch(SizeMismatch {
                expected: file_size,
                actual: actual_size,
            }));
        }

        Ok(())
    }
}
