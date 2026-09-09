# Test Plan

- Round-trip the versioned JSON document.
- Recover from absent, corrupt, malformed, and future-version documents.
- Verify debounced updates and explicit flush.
- Verify project isolation and the recent-content limit.
- Verify sidebar/drawer enum normalization and panel dimension bounds.
- Verify Settings page/provider restoration normalization.
- Verify window placement stays on a current display.
- Verify stale session/dependency/content references fall back safely.
- Run desktop tests, desktop build, design-system build, and `git diff --check`.
