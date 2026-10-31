


use crate::model::Parsed;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

const MAX_ENTRIES: usize = 16;
/// Upper bound on the summed source sizes of cached entries.
const MAX_SOURCE_BYTES: usize = 128 * 1024 * 1024;

struct Entry {
    path: PathBuf,
    fingerprint: String,
    parse: Arc<Parsed>,
    source_bytes: usize,
}

#[derive(Default)]
pub(crate) struct Cache {
    entries: Mutex<Vec<Entry>>,
}

impl Cache {
    pub(crate) fn get(&self, path: &Path, fingerprint: &str) -> Option<Arc<Parsed>> {
        let mut entries = self.entries.lock().ok()?;
        let position = entries
            .iter()
            .position(|entry| entry.path == path && entry.fingerprint == fingerprint)?;
        let entry = entries.remove(position);
        let parse = entry.parse.clone();
        entries.push(entry);
        Some(parse)
    }

    pub(crate) fn insert(
        &self,
        path: &Path,
        fingerprint: &str,
        parse: Arc<Parsed>,
        source_bytes: usize,
    ) {
        let Ok(mut entries) = self.entries.lock() else {
            return;
        };
        entries.retain(|entry| entry.path != path);
        entries.push(Entry {
            path: path.to_path_buf(),
            fingerprint: fingerprint.to_string(),
            parse,
            source_bytes,
        });
        while entries.len() > 1
            && (entries.len() > MAX_ENTRIES
                || entries
                    .iter()
                    .map(|entry| entry.source_bytes)
                    .sum::<usize>()
                    > MAX_SOURCE_BYTES)
        {
            entries.remove(0);
        }
    }
}
