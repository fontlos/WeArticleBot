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

// 用于匹配一个字符串中的某个标签对之间的内容
// 右空则匹配到底, 左空报错, 因为暂时没必要有这个功能
pub fn parse_by_tag<'a>(bytes: &'a [u8], left: &str, right: &str) -> Option<&'a str> {
    let left = left.as_bytes();
    let right = right.as_bytes();
    let start = bytes.windows(left.len()).position(|w| w == left)?;
    let start = start + left.len();
    if right.is_empty() {
        return std::str::from_utf8(&bytes[start..]).ok();
    }
    let rest = &bytes[start..];
    let end = rest.windows(right.len()).position(|w| w == right)?;
    std::str::from_utf8(&rest[..end]).ok()
}
