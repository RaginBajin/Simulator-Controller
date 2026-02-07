use std::path::Path;

use tracing::error;

use crate::parquet::checksum as parquet_checksum;

/// Validate a Parquet file's checksum and return (is_valid, actual_checksum).
pub fn validate_checksum_with_details(
    file_path: &Path,
    expected_checksum: &str,
    session_id: &str,
) -> Result<(bool, String), crate::error::StorageError> {
    let actual = parquet_checksum::compute_checksum(file_path)?;
    let is_valid = actual == expected_checksum;

    if !is_valid {
        error!(
            session_id = session_id,
            expected = expected_checksum,
            actual = actual,
            path = %file_path.display(),
            "Checksum mismatch detected"
        );
    }

    Ok((is_valid, actual))
}
