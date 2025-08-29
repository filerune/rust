## 0.2.0

### Breaking Changes

- Remove `CheckResult` struct
- Remove `CheckResultErrorType` enum

### What's New

- Add support for `smol` (require `smol` feature)
- Add `MissingChunks` struct
- Add `SizeMismatch` struct

### What's Changed

- Extend `CheckError` enum

### Migrating from 0.1.0 to 0.2.0

from:

```rust
use std::path::PathBuf;

use filerune_fusion::check::{Check, CheckResult};

let result: CheckResult = Check::new()
    .in_dir(PathBuf::from("path").join("to").join("dir"))
    .file_size(0)
    .total_chunks(0)
    .run()
    .unwrap();
```

to:

```rust
use std::path::PathBuf;

use filerune_fusion::check::{Check, CheckError};

if let Err(error) = Check::new()
    .in_dir(PathBuf::from("path").join("to").join("dir"))
    .file_size(0)
    .total_chunks(0)
    .run()
{
    match error {
        CheckError::Missing(err) => {
            // ...
        },
        CheckError::Size(err) => {
            // ...
        },
        _ => {
            // ...
        }
    }
}
```

## 0.1.0 (2025-06-09)

First release
