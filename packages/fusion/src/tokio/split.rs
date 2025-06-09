use std::path::{Path, PathBuf};

use tokio::{
    fs,
    io::{self, AsyncReadExt as _, AsyncWriteExt as _},
};

use crate::split::{Split, SplitError, SplitResult};

/// Trait for running the split process.
pub trait SplitAsyncExt {
    /// Run the split process asynchronously.
    fn run_async(
        &self
    ) -> impl std::future::Future<Output = Result<SplitResult, SplitError>> + Send;
}

impl SplitAsyncExt for Split {
    async fn run_async(&self) -> Result<SplitResult, SplitError> {
        let in_file: &Path = match self.in_file {
            | Some(ref p) => {
                let p: &Path = p.as_ref();

                // if in_file not exists
                if !p.exists() {
                    return Err(SplitError::InFileNotFound);
                }

                // if in_file not a file
                if !p.is_file() {
                    return Err(SplitError::InFileNotFile);
                }

                p
            },
            | None => return Err(SplitError::InFileNotSet),
        };

        let out_dir: &Path = match self.out_dir {
            | Some(ref p) => {
                let p: &Path = p.as_ref();

                if !p.exists() {
                    // if out_dir not exists
                    fs::create_dir_all(p)
                        .await
                        .map_err(|_| SplitError::OutDirNotCreated)?
                } else if p.is_file() {
                    // if out_dir not a directory
                    return Err(SplitError::OutDirNotDir);
                }

                p
            },
            | None => return Err(SplitError::OutDirNotSet),
        };

        let chunk_size: usize = self.chunk_size;

        let buffer_capacity: usize = self.buffer_capacity;

        let input_file: fs::File = fs::OpenOptions::new()
            .read(true)
            .open(in_file)
            .await
            .map_err(|_| SplitError::InFileNotOpened)?;

        let file_size: usize = input_file
            .metadata()
            .await
            .map_err(|_| SplitError::InFileNotRead)?
            .len() as usize;

        let mut reader: io::BufReader<fs::File> =
            io::BufReader::with_capacity(buffer_capacity, input_file);

        let mut buffer: Vec<u8> = vec![0; chunk_size];

        let mut total_chunks: usize = 0;

        loop {
            let mut offset: usize = 0;

            while offset < chunk_size {
                match reader.read(&mut buffer[offset..]).await {
                    | Ok(0) => break,
                    | Ok(n) => offset += n,
                    | Err(_) => return Err(SplitError::InFileNotRead),
                };
            }

            if offset == 0 {
                break;
            }

            let output_path: PathBuf = out_dir.join(total_chunks.to_string());

            let output: fs::File = fs::OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(output_path)
                .await
                .map_err(|_| SplitError::OutFileNotOpened)?;

            let mut writer: io::BufWriter<fs::File> =
                io::BufWriter::with_capacity(buffer_capacity, output);

            writer
                .write_all(&buffer[..offset])
                .await
                .map_err(|_| SplitError::OutFileNotWritten)?;

            writer.flush().await.map_err(|_| SplitError::OutFileNotWritten)?;

            total_chunks += 1;
        }

        Ok(SplitResult { file_size, total_chunks })
    }
}
