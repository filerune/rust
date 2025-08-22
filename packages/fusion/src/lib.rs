//! # FileRune Fusion
//!
//! A file splitting & merging solution.
//!
//! ## Quick Start
//!
//! Split file from a path to a directory with `Split` struct.
//!
//! ```no_run
//! use std::path::PathBuf;
//!
//! use filerune_fusion::split::{Split, SplitResult};
//!
//! let result: SplitResult = Split::new()
//!     .in_file(PathBuf::from("path").join("to").join("file"))
//!     .out_dir(PathBuf::from("path").join("to").join("dir"))
//!     .run()
//!     .unwrap();
//! ```
//!
//! Async version also available with the `async_std` and `tokio` features:
//!
//! ```no_run
//! // This is a `async_std` example
//!
//! use async_std::path::PathBuf;
//!
//! use filerune_fusion::split::{
//!     Split,
//!     SplitResult,
//!     async_std::SplitAsyncExt as _,
//! };
//!
//! # async fn example() {
//! let result: SplitResult = Split::new()
//!     .in_file(PathBuf::from("path").join("to").join("file"))
//!     .out_dir(PathBuf::from("path").join("to").join("dir"))
//!     .run_async()
//!     .await
//!     .unwrap();
//! # }
//! ```
//!
//! ```no_run
//! // This is a `tokio` example
//!
//! use std::path::PathBuf;
//!
//! use filerune_fusion::split::{
//!     Split,
//!     SplitResult,
//!     tokio::SplitAsyncExt as _,
//! };
//!
//! # async fn example() {
//! let result: SplitResult = Split::new()
//!     .in_file(PathBuf::from("path").join("to").join("file"))
//!     .out_dir(PathBuf::from("path").join("to").join("dir"))
//!     .run_async()
//!     .await
//!     .unwrap();
//! # }
//! ```

/// Split module.
pub mod split;

/// Check module.
pub mod check;

/// Merge module.
pub mod merge;

/// Functions implemented with `async_std`.
#[cfg(feature = "async_std")]
pub(crate) mod async_std;

/// Functions implemented with `smol`.
#[cfg(feature = "smol")]
pub(crate) mod smol;

/// Functions implemented with `tokio`.
#[cfg(feature = "tokio")]
pub(crate) mod tokio;

/// The default chunk size in bytes.
pub const CHUNK_SIZE_DEFAULT: usize = 2 * 1024 * 1024;

/// The default buffer capacity in bytes.
pub const BUFFER_CAPACITY_DEFAULT: usize = 1024 * 1024;
