# Test Plan

## Scope

Verify embedded repository discovery, structured status/diff semantics, SDK ownership, and the Qt drawer/footer presentation.

## Unit tests

- Clean, modified, staged, staged-and-modified, untracked, renamed, deleted, binary, and conflict states.
- All, staged, and unstaged diff scopes.
- Unborn repository and non-repository errors.
- Path boundary validation and bounded/truncated output.

## Integration and conformance tests

- Runtime SDK methods resolve projects and return stable DTO shapes.
- C ABI methods serialize success and typed failures.
- Shared vectors cover Git status and file diff results.

## Regression checks

- Existing operations and runtime tests.
- Qt desktop build and QML lint.
- Project-window footer model and session token content.

## Manual checks

- Toggle the drawer from the left gutter and footer.
- Resize the drawer and window at 1440x900 and 900x620.
- Inspect clean, dirty, untracked, binary, conflict, non-repository, loading, and error states in dark and light modes.
- Verify keyboard focus and readable non-color status cues.

## Commands and results

- `cargo test --workspace` passed: 17 operations tests and 26 runtime tests.
- `cmake --build apps/desktop-qt/build -j2` passed, including the embedded Rust static library and QML cache compilation.
- `cmake --build apps/desktop-qt/build --target all_qmllint -j2` passed with the repository's existing import-resolution and unqualified-access warnings.
- Focused `qmllint` reported no new missing-property or incompatible-type diagnostics; the unresolved `SunCode.Runtime` standalone import diagnostics remain existing behavior.
- Impeccable detector passed for the new drawer and changed project window.
- `jq empty contracts/vectors/runtime-sdk.json` passed.
- Offscreen startup remained in the Qt event loop without QML runtime errors.
- Native clean and dirty repository checks passed at 1440x900 and 900x620; the latter exposed and verified a footer-reservation fix.
- `git diff --check` passed.

## Residual risks

- Physical pointer verification of the resize grip and keyboard traversal remains manual.
- Vendored Rust objects currently produce existing macOS deployment-target mismatch warnings during linking; the desktop binary still links successfully.
- Git filters, attributes, submodules, and unusual encodings can differ from Git CLI behavior outside the bounded review contract.
- Git mutations and remote operations remain deferred to separate policy, credentials, recovery, and undo work.
