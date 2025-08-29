use std::{
    fs,
    path::{Path, PathBuf},
};

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
    pub use crate::async_std::check::CheckAsyncExt;
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
    pub use crate::smol::check::CheckAsyncExt;
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
    pub use crate::tokio::check::CheckAsyncExt;
}

/// Check process missing chunks error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingChunks {
    // pub missing: Vec<usize>,
    pub missing: Vec<usize>,
}

/// Check process size mismatch error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SizeMismatch {
    pub expected: usize,
    pub actual: usize,
}

/// Check process error enum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckError {
    InDirNotFound,
    InDirNotDir,
    InDirNotSet,
    InFileNotOpened,
    InFileNotRead,
    FileSizeNotSet,
    TotalChunksNotSet,
    MissingChunks(MissingChunks),
    SizeMismatch(SizeMismatch),
}

impl CheckError {
    /// Get the code of the error as `&str`.
    pub fn as_code(&self) -> &str {
        match self {
            | Self::InDirNotFound => "in_dir_not_found",
            | Self::InDirNotDir => "in_dir_not_dir",
            | Self::InDirNotSet => "in_dir_not_set",
            | Self::InFileNotOpened => "in_file_not_opened",
            | Self::InFileNotRead => "in_file_not_read",
            | Self::FileSizeNotSet => "file_size_not_set",
            | Self::TotalChunksNotSet => "total_chunks_not_set",
            | Self::MissingChunks(_) => "missing_chunks",
            | Self::SizeMismatch(_) => "size_mismatch",
        }
    }

    /// Get the code of the error as `String`.
    pub fn to_code(&self) -> String {
        self.as_code().to_string()
    }

    /// Get the message of the error as `&str`.
    pub fn as_message(&self) -> &str {
        match self {
            | Self::InDirNotFound => "The input directory not found.",
            | Self::InDirNotDir => "The input directory is not a directory.",
            | Self::InDirNotSet => "The input directory is not set.",
            | Self::InFileNotOpened => "The input file could not be opened.",
            | Self::InFileNotRead => "The input file could not be read.",
            | Self::FileSizeNotSet => "The `file_size` is not set.",
            | Self::TotalChunksNotSet => "The `total_chunks` is not set.",
            | Self::MissingChunks(_) => {
                "Some of the chunks are missing to merge the file."
            },
            | Self::SizeMismatch(_) => {
                "The actual file size is not equal the input file size."
            },
        }
    }

    /// Get the message of the error as `String`.
    pub fn to_message(&self) -> String {
        self.as_message().to_string()
    }
}

/// Process to check the file integrity.
///
/// ## Example
///
/// ```no_run
/// use std::path::PathBuf;
///
/// use filerune_fusion::check::{Check, CheckError};
///
/// if let Err(error) = Check::new()
///     .in_dir(PathBuf::from("path").join("to").join("dir"))
///     .file_size(0) // result from split function...
///     .total_chunks(0) // result from split function...
///     .run()
/// {
///     match error {
///         | CheckError::MissingChunks(err) => {
///             // ...
///         },
///         | CheckError::SizeMismatch(err) => {
///             // ...
///         },
///         | _ => {
///             // ...
///         },
///     }
/// };
/// ```
#[derive(Debug, Clone)]
pub struct Check {
    pub in_dir: Option<PathBuf>,
    pub file_size: Option<usize>,
    pub total_chunks: Option<usize>,
}

impl Check {
    /// Create a new check process.
    pub fn new() -> Self {
        Self { in_dir: None, file_size: None, total_chunks: None }
    }

    /// Create a new check process from an existing one.
    pub fn from<P: Into<Check>>(process: P) -> Self {
        process.into()
    }

    /// Set the input directory.
    pub fn in_dir<InDir: AsRef<Path>>(
        mut self,
        path: InDir,
    ) -> Self {
        self.in_dir = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set the size of the original file in bytes.
    pub fn file_size(
        mut self,
        size: usize,
    ) -> Self {
        self.file_size = Some(size);
        self
    }

    /// Set the total number of chunks splitted from the original file.
    pub fn total_chunks(
        mut self,
        chunks: usize,
    ) -> Self {
        self.total_chunks = Some(chunks);
        self
    }

    /// Run the check process.
    pub fn run(&self) -> Result<bool, CheckError> {
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

            let file: fs::File =
                match fs::OpenOptions::new().read(true).open(&target_file) {
                    | Ok(f) => f,
                    | Err(_) => {
                        missing.push(i);
                        continue;
                    },
                };

            let metadata: fs::Metadata =
                file.metadata().map_err(|_| CheckError::InFileNotRead)?;

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

        Ok(true)
    }
}

impl Default for Check {
    fn default() -> Self {
        Self::new()
    }
}
