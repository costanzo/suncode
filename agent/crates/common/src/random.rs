//! Cryptographically secure random values from the operating system.
//!
//! Use this for keys, nonces, tokens, and jitter; use [`crate::id`] for ids.

/// Fills `buffer` from the OS secure random source.
///
/// # Panics
///
/// If the OS random source is unavailable. Continuing without one would
/// produce predictable secrets, so this is treated as a fatal host fault.
pub fn fill(buffer: &mut [u8]) {
    getrandom::fill(buffer).expect("the operating system's secure random source is unavailable");
}

/// `N` secure random bytes.
pub fn bytes<const N: usize>() -> [u8; N] {
    let mut buffer = [0u8; N];
    fill(&mut buffer);
    buffer
}

/// A secure random `u64`.
pub fn u64() -> u64 {
    u64::from_le_bytes(bytes())
}

/// A uniformly distributed value in `0..bound`, without modulo bias.
///
/// # Panics
///
/// If `bound` is zero.
pub fn below(bound: u64) -> u64 {
    assert!(bound > 0, "random bound must be positive");
    // Reject the top partial range so every residue is equally likely.
    let zone = u64::MAX - (u64::MAX % bound);
    loop {
        let value = u64();
        if value < zone {
            return value % bound;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_are_filled_and_differ_between_calls() {
        let first = bytes::<32>();
        let second = bytes::<32>();
        assert_ne!(first, [0; 32]);
        assert_ne!(first, second);
    }

    #[test]
    fn below_stays_in_range() {
        assert!((0..1_000).all(|_| below(251) < 251));
        assert_eq!(below(1), 0);
    }
}
