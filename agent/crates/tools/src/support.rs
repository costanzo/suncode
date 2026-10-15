//! Small helpers shared across operations.

use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Seconds since the Unix epoch, as a decimal string.
pub(crate) fn now_string() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", seconds)
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
