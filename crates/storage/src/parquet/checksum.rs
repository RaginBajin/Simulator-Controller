use std::fs::File;
use std::io::Read;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::error::StorageError;

/// Compute the SHA-256 checksum of a file, returned as a hex string.
pub fn compute_checksum(path: &Path) -> Result<String, StorageError> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Validate a file's SHA-256 checksum against an expected value.
pub fn validate_checksum(path: &Path, expected: &str) -> Result<bool, StorageError> {
    let actual = compute_checksum(path)?;
    Ok(actual == expected)
}
