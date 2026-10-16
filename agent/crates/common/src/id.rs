//! Identifier generation.
//!
//! An id is `<prefix>_<26 Crockford base32 characters>` encoding a UUID v7:
//! a 48-bit Unix millisecond timestamp followed by random bits. Ids are
//! URL- and filename-safe, sort by creation time as plain strings, and are
//! ordered within a process even when created in the same millisecond. The
//! prefix names the kind, so an id is identifiable in logs and events.
//!
//! Ids are opaque: nothing may parse them, and stores also hold ids written
//! before this scheme (bare UUIDs). Use [`crate::random`] for secrets.

use uuid::Uuid;

/// Lowercase Crockford base32: no `i`, `l`, `o`, or `u`, and ASCII order
/// matches numeric order so encoded ids sort like the values they encode.
const ALPHABET: &[u8; 32] = b"0123456789abcdefghjkmnpqrstvwxyz";
/// 128 bits in 5-bit groups; the first character carries the top 3 bits.
const ENCODED_LEN: usize = 26;

/// The kinds of generated ids and their prefixes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdKind {
    Project,
    Dependency,
    Session,
    Turn,
    Message,
    /// One model provider call.
    Exchange,
    Approval,
    Question,
    CheckpointManifest,
    QueuedMessage,
    SubagentInvocation,
    /// Internal submission key for turns the agent starts itself.
    Submission,
    /// Remote Server request.
    RemoteRequest,
    /// Remote Server event.
    RemoteEvent,
}

impl IdKind {
    pub const fn prefix(self) -> &'static str {
        match self {
            Self::Project => "prj",
            Self::Dependency => "dep",
            Self::Session => "ses",
            Self::Turn => "trn",
            Self::Message => "msg",
            Self::Exchange => "xch",
            Self::Approval => "apr",
            Self::Question => "que",
            Self::CheckpointManifest => "ckm",
            Self::QueuedMessage => "qmsg",
            Self::SubagentInvocation => "sub",
            Self::Submission => "sbm",
            Self::RemoteRequest => "rreq",
            Self::RemoteEvent => "revt",
        }
    }
}

/// A new id of `kind`.
pub fn new(kind: IdKind) -> String {
    with_prefix(kind.prefix())
}

/// A new id with a caller-chosen prefix, for names scoped to one component
/// (for example `browser` artifacts). The prefix must be lowercase ASCII
/// letters or digits.
pub fn with_prefix(prefix: &str) -> String {
    debug_assert!(
        !prefix.is_empty()
            && prefix
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit()),
        "id prefix must be lowercase alphanumeric: {prefix:?}"
    );
    let mut id = String::with_capacity(prefix.len() + 1 + ENCODED_LEN);
    id.push_str(prefix);
    id.push('_');
    encode(Uuid::now_v7().as_u128(), &mut id);
    id
}

fn encode(value: u128, output: &mut String) {
    for index in (0..ENCODED_LEN).rev() {
        let digit = (value >> (index * 5)) & 0x1f;
        output.push(char::from(ALPHABET[digit as usize]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn ids_have_the_prefix_and_a_fixed_length_base32_body() {
        let id = new(IdKind::Session);
        let (prefix, body) = id.split_once('_').unwrap();
        assert_eq!(prefix, "ses");
        assert_eq!(body.len(), ENCODED_LEN);
        assert!(body.bytes().all(|byte| ALPHABET.contains(&byte)));
        assert!(body.starts_with(['0', '1', '2', '3', '4', '5', '6', '7']));
    }

    #[test]
    fn ids_are_unique_and_sort_in_creation_order() {
        let ids = (0..2_000).map(|_| new(IdKind::Message)).collect::<Vec<_>>();
        assert_eq!(ids.iter().collect::<HashSet<_>>().len(), ids.len());
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(sorted, ids);
    }

    #[test]
    fn encoding_preserves_numeric_order_and_known_values() {
        let mut zero = String::new();
        encode(0, &mut zero);
        assert_eq!(zero, "0".repeat(ENCODED_LEN));
        let mut max = String::new();
        encode(u128::MAX, &mut max);
        assert_eq!(max, format!("7{}", "z".repeat(ENCODED_LEN - 1)));
        let mut low = String::new();
        let mut high = String::new();
        encode(31, &mut low);
        encode(32, &mut high);
        assert!(low < high);
    }

    #[test]
    fn prefixes_are_distinct_and_lowercase() {
        let kinds = [
            IdKind::Project,
            IdKind::Dependency,
            IdKind::Session,
            IdKind::Turn,
            IdKind::Message,
            IdKind::Exchange,
            IdKind::Approval,
            IdKind::Question,
            IdKind::CheckpointManifest,
            IdKind::QueuedMessage,
            IdKind::SubagentInvocation,
            IdKind::Submission,
            IdKind::RemoteRequest,
            IdKind::RemoteEvent,
        ];
        let prefixes = kinds.map(IdKind::prefix);
        assert_eq!(prefixes.iter().collect::<HashSet<_>>().len(), kinds.len());
        assert!(prefixes
            .iter()
            .all(|prefix| prefix.bytes().all(|byte| byte.is_ascii_lowercase())));
    }
}
