use rand::RngExt;

use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{Error, Result};

pub fn timestamp() -> Result<u64> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| Error::Custom(format!("Failed to get timestamp: {}", e)))?
        .as_secs();
    Ok(timestamp)
}

pub fn random_string(len: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    (0..len)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
