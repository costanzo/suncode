use super::*;

impl AsyncAgentSdk {
    /// Net diff of every file `write`, `edit`, and `delete` changed during one turn.
    pub fn turn_diff(&self, session_id: &str, turn_id: &str) -> SdkResult<TurnDiff> {
        self.session_for_user(session_id)?;
        let mut files = Vec::new();
        for version in self.state.store.turn_file_versions(session_id, turn_id)? {
            fn stored(version: &Option<suncode_data::StoredFileVersion>) -> Option<(&str, &str)> {
                version
                    .as_ref()
                    .map(|v| (v.stored_on.as_str(), v.sha256.as_str()))
            }
            let diff = self
                .state
                .operations
                .checkpoint_diff(stored(&version.baseline), stored(&version.current))?;
            files.push(TurnFileDiff {
                status: version.status().to_string(),
                path: version.path,
                additions: diff.additions,
                deletions: diff.deletions,
                binary: diff.binary,
                hunks: diff.hunks,
            });
        }
        Ok(TurnDiff { 
            session_id: session_id.to_string(),
            turn_id: turn_id.to_string(),
            files,
        })
    }
}

/// Drops expired turn changes, then the day folders of stored file versions
/// none of the remaining changes can reference. Runs off the caller's thread.
pub(crate) fn retain_turn_changes(store: Store, operations: Arc<suncode_tool::Operations>) {
    std::thread::spawn(move || match store.retain_turn_changes() {
        Ok(oldest_kept_day) => operations.retain_checkpoints(&oldest_kept_day),
        Err(error) => logging::write_business_error("turn_diff", "retain", &error, ""),
    });
}