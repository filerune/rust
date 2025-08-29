use std::{
    fs,
    io::{self, Read as _, Write as _},
    path::{Path, PathBuf},
};

use crate::{BUFFER_CAPACITY_DEFAULT, CHUNK_SIZE_DEFAULT};

/// Run asynchronously with `async_std` feature.
///
/// To use it, add the following code to the `Cargo.toml` file:
///
/// ```toml
/// [dependencies]
/// filerune_fusion = { version = "*", features = ["async_std"] }
/// ```
#[cfg(feature = "async_std")]
pub mod async_std {
    pub use crate::async_std::split::SplitAsyncExt;
}

/// Run asynchronously with `smol` feature.
///
/// To use it, add the following code to the `Cargo.toml` file:
///
/// ```toml
/// [dependencies]
/// filerune_fusion = { version = "*", features = ["smol"] }
/// ```
#[cfg(feature = "smol")]
pub mod smol {
    pub use crate::smol::split::SplitAsyncExt;
}

/// Run asynchronously with `tokio` feature.
///
/// To use it, add the following code to the `Cargo.toml` file:
///
/// ```toml
/// [dependencies]
/// filerune_fusion = { version = "*", features = ["tokio"] }
/// ```
#[cfg(feature = "tokio")]
pub mod tokio {
    pub use crate::tokio::split::SplitAsyncExt;
}

/// Result of the split process.
#[derive(Debug, Clone)]
pub struct SplitResult {
    /// Size of the original file in bytes.
    pub file_size: usize,
    /// The total number of chunks splitted from the original file.
    pub total_chunks: usize,
}

/// Split process error enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitError {
    InFileNotFound,
    InFileNotFile,
    InFileNotSet,
    InFileNotOpened,
    InFileNotRead,
    OutDirNotCreated,
    OutDirNotDir,
    OutDirNotSet,
    OutFileNotOpened,
    OutFileNotWritten,
}

impl SplitError {
    /// Get the code of the error as `&str`.
    pub fn as_code(&self) -> &str {
        match self {
            | Self::InFileNotFound => "in_file_not_found",
            | Self::InFileNotFile => "in_file_not_file",
            | Self::InFileNotSet => "in_file_not_set",
            | Self::InFileNotOpened => "in_file_not_opened",
            | Self::InFileNotRead => "in_file_not_read",
            | Self::OutDirNotCreated => "out_dir_not_created",
            | Self::OutDirNotDir => "out_dir_not_dir",
            | Self::OutDirNotSet => "out_dir_not_set",
            | Self::OutFileNotOpened => "out_file_not_opened",
            | Self::OutFileNotWritten => "out_file_not_written",
        }
    }

    /// Get the code of the error as `String`.
    pub fn to_code(&self) -> String {
        self.as_code().to_string()
    }

    /// Get the message of the error as `&str`.
    pub fn as_message(&self) -> &str {
        match self {
            | Self::InFileNotFound => "The input file not found.",
            | Self::InFileNotFile => "The input file is not a file.",
            | Self::InFileNotSet => "The input file is not set.",
            | Self::InFileNotOpened => "The input file could not be opened.",
            | Self::InFileNotRead => "The input file could not be read.",
            | Self::OutDirNotCreated => {
                "The output directory could not be created."
            },
            | Self::OutDirNotDir => "The output directory is not a directory.",
            | Self::OutDirNotSet => "The output directory is not set.",
            | Self::OutFileNotOpened => {
                "The output file could not be created or opened."
            },
            | Self::OutFileNotWritten => {
                "The output file could not be written."
            },
        }
    }

    /// Get the message of the error as `String`.
    pub fn to_message(&self) -> String {
        self.as_message().to_string()
    }
}

/// Process to split file from a path to a directory.
///
/// ## Example
///
/// ```no_run
/// use std::path::PathBuf;
///
/// use filerune_fusion::split::{Split, SplitResult};
///
/// let result: SplitResult = Split::new()
///     .in_file(PathBuf::from("path").join("to").join("file"))
///     .out_dir(PathBuf::from("path").join("to").join("dir"))
///     .run()
///     .unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct Split {
    pub in_file: Option<PathBuf>,
    pub out_dir: Option<PathBuf>,
    pub chunk_size: usize,
    pub buffer_capacity: usize,
}

impl Split {
    /// Create a new split process.
    pub fn new() -> Self {
        Self {
            in_file: None,
            out_dir: None,
            chunk_size: CHUNK_SIZE_DEFAULT,
            buffer_capacity: BUFFER_CAPACITY_DEFAULT,
        }
    }

    /// Create a new split process from an existing one.
    pub fn from<P: Into<Split>>(process: P) -> Self {
        process.into()
    }

    /// Set the input file.
    pub fn in_file<InFile: AsRef<Path>>(
        mut self,
        path: InFile,
    ) -> Self {
        self.in_file = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set the output directory.
    pub fn out_dir<OutDir: AsRef<Path>>(
        mut self,
        path: OutDir,
    ) -> Self {
        self.out_dir = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set the maximum size of each chunk.
    ///
    /// By default, the chunk size follows the [`CHUNK_SIZE_DEFAULT`].
    pub fn chunk_size(
        mut self,
        size: usize,
    ) -> Self {
        self.chunk_size = size;
        self
    }

    /// Set the size of the buffer capacity.
    ///
    /// By default, it is [`BUFFER_CAPACITY_DEFAULT`].
    pub fn buffer_capacity(
        mut self,
        capacity: usize,
    ) -> Self {
        self.buffer_capacity = capacity;
        self
    }

    /// Run the split process.
    pub fn run(&self) -> Result<SplitResult, SplitError> {
        let in_file: &Path = match self.in_file {
            | Some(ref p) => {
                let p: &Path = p.as_path();

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
                let p: &Path = p.as_path();

                if !p.exists() {
                    // if out_dir not exists
                    fs::create_dir_all(p)
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
            .map_err(|_| SplitError::InFileNotOpened)?;

        let file_size: usize =
            input_file.metadata().map_err(|_| SplitError::InFileNotRead)?.len()
                as usize;

        let mut reader: io::BufReader<fs::File> =
            io::BufReader::with_capacity(buffer_capacity, input_file);

        let mut buffer: Vec<u8> = vec![0; chunk_size];

        let mut total_chunks: usize = 0;

        loop {
            let mut offset: usize = 0;

            while offset < chunk_size {
                match reader.read(&mut buffer[offset..]) {
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
                .map_err(|_| SplitError::OutFileNotOpened)?;

            let mut writer: io::BufWriter<fs::File> =
                io::BufWriter::with_capacity(buffer_capacity, output);

            writer
                .write_all(&buffer[..offset])
                .map_err(|_| SplitError::OutFileNotWritten)?;

            writer.flush().map_err(|_| SplitError::OutFileNotWritten)?;

            total_chunks += 1;
        }

        Ok(SplitResult { file_size, total_chunks })
    }
}

impl Default for Split {
    fn default() -> Self {
        Self::new()
    }
}
