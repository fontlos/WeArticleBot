use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{Error, Result};

pub fn timestamp() -> Result<u64> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| Error::Custom(format!("Failed to get timestamp: {}", e)))?
        .as_secs();
    Ok(timestamp)
}
