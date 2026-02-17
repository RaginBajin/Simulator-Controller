use std::fs;
use std::path::Path;

use tracing::warn;

use crate::error::StorageError;

/// Scan the telemetry directory for orphaned `.parquet.tmp` files and remove them.
///
/// Returns the number of files cleaned up. Called on startup to recover from crashes.
pub fn cleanup_orphaned_temps(telemetry_dir: &Path) -> Result<u32, StorageError> {
    if !telemetry_dir.exists() {
        return Ok(0);
    }

    let mut cleaned = 0u32;

    for entry in fs::read_dir(telemetry_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Check for .tmp extension on files that look like parquet temps
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".parquet.tmp") {
                    warn!("Removing orphaned temp file: {}", path.display());
                    fs::remove_file(&path)?;
                    cleaned += 1;
                }
            }
        }
    }

    Ok(cleaned)
}
