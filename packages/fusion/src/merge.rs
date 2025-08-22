use std::{
    fs,
    io::{self, Read as _, Write as _},
    path::{Path, PathBuf},
};

use crate::BUFFER_CAPACITY_DEFAULT;

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
    pub use crate::async_std::merge::MergeAsyncExt;
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
    pub use crate::smol::merge::MergeAsyncExt;
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
    pub use crate::tokio::merge::MergeAsyncExt;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeError {
    InDirNotFound,
    InDirNotDir,
    InDirNotSet,
    InDirNotRead,
    InDirNoFile,
    InFileNotOpened,
    InFileNotRead,
    OutDirNotCreated,
    OutFileNotSet,
    OutFileNotRemoved,
    OutFileNotOpened,
    OutFileNotWritten,
}

impl MergeError {
    /// Get the code of the error as `&str`.
    pub fn as_code(&self) -> &str {
        match self {
            | Self::InDirNotFound => "in_dir_not_found",
            | Self::InDirNotDir => "in_dir_not_dir",
            | Self::InDirNotSet => "in_dir_not_set",
            | Self::InDirNotRead => "in_dir_not_read",
            | Self::InDirNoFile => "in_dir_no_file",
            | Self::InFileNotOpened => "in_file_not_opened",
            | Self::InFileNotRead => "in_file_not_read",
            | Self::OutDirNotCreated => "out_dir_not_created",
            | Self::OutFileNotSet => "out_file_not_set",
            | Self::OutFileNotRemoved => "out_file_not_removed",
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
            | Self::InDirNotFound => "The input directory not found.",
            | Self::InDirNotDir => "The input directory is not a directory.",
            | Self::InDirNotSet => "The input directory is not set.",
            | Self::InDirNotRead => "The input directory could not be read.",
            | Self::InDirNoFile => "The input directory has no file.",
            | Self::InFileNotOpened => "The input file could not be opened.",
            | Self::InFileNotRead => "The input file could not be read.",
            | Self::OutDirNotCreated => {
                "The output directory could not be created."
            },
            | Self::OutFileNotSet => "The output file is not set.",
            | Self::OutFileNotRemoved => {
                "The output file could not be removed."
            },
            | Self::OutFileNotOpened => "The output file could not be opened.",
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

/// Process to merge chunks from a directory to a path.
///
/// ## Example
///
/// ```no_run
/// use std::path::PathBuf;
///
/// use filerune_fusion::merge::Merge;
///
/// let result: bool = Merge::new()
///     .in_dir(PathBuf::from("path").join("to").join("dir"))
///     .out_file(PathBuf::from("path").join("to").join("file"))
///     .run()
///     .unwrap();
/// ```
#[derive(Debug, Clone)]
pub struct Merge {
    pub in_dir: Option<PathBuf>,
    pub out_file: Option<PathBuf>,
    pub buffer_capacity: usize,
}

impl Merge {
    /// Create a new merge process.
    pub fn new() -> Self {
        Self {
            in_dir: None,
            out_file: None,
            buffer_capacity: BUFFER_CAPACITY_DEFAULT,
        }
    }

    /// Create a new merge process from an existing one.
    pub fn from<P: Into<Merge>>(process: P) -> Self {
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

    /// Set the output file.
    pub fn out_file<OutFile: AsRef<Path>>(
        mut self,
        path: OutFile,
    ) -> Self {
        self.out_file = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set the maximum size of the buffer capacity.
    ///
    /// By default, it is [`BUFFER_CAPACITY_DEFAULT`].
    pub fn buffer_capacity(
        mut self,
        capacity: usize,
    ) -> Self {
        self.buffer_capacity = capacity;
        self
    }

    /// Run the merge process.
    pub fn run(&self) -> Result<bool, MergeError> {
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

                // delete out_path target if exists
                if p.exists() {
                    if p.is_dir() {
                        fs::remove_dir_all(p)
                            .map_err(|_| MergeError::OutFileNotRemoved)?;
                    } else {
                        fs::remove_file(p)
                            .map_err(|_| MergeError::OutFileNotRemoved)?;
                    }
                }

                // create outpath
                if let Some(parent) = p.parent() {
                    fs::create_dir_all(parent)
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
            .map_err(|_| MergeError::OutFileNotOpened)?;

        // writer
        let mut writer: io::BufWriter<fs::File> =
            io::BufWriter::with_capacity(buffer_capacity, output);

        // get inputs
        let mut entries: Vec<PathBuf> = {
            let read_dir: fs::ReadDir =
                fs::read_dir(in_dir).map_err(|_| MergeError::InDirNotRead)?;

            read_dir
                .filter_map(Result::ok)
                .filter(|entry| entry.path().is_file())
                .map(|entry| entry.path())
                .collect()
        };

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
                .map_err(|_| MergeError::InFileNotOpened)?;

            let mut reader: io::BufReader<fs::File> =
                io::BufReader::with_capacity(buffer_capacity, input);

            let mut buffer: Vec<u8> = vec![0; buffer_capacity];

            loop {
                let read: usize = reader
                    .read(&mut buffer)
                    .map_err(|_| MergeError::InFileNotRead)?;

                if read == 0 {
                    break;
                }

                writer
                    .write_all(&buffer[..read])
                    .map_err(|_| MergeError::OutFileNotWritten)?;
            }
        }

        writer.flush().map_err(|_| MergeError::OutFileNotWritten)?;

        Ok(true)
    }
}

impl Default for Merge {
    fn default() -> Self {
        Self::new()
    }
}
