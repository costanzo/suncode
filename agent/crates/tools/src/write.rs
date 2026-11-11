use crate::arguments::{DeleteArguments, WriteArguments};
use crate::checkpoint;
use crate::error::{fatal, retryable};
use crate::scope::safe_relative_path;
use crate::support::sha256_hex;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use suncode_common::BusinessError;

// All built-in mutations share this gate, including separate Operations hosts.
// External writers do not participate: the final digest check is optimistic.
static MUTATIONS: Mutex<()> = Mutex::new(());

#[cfg(windows)]
mod windows;

pub(crate) fn write(
    project_root: Option<&Path>,
    checkpoint_root: Option<&Path>,
    args: &WriteArguments,
) -> Result<Value, BusinessError> {
    let bytes = STANDARD
        .decode(&args.content_base64)
        .map_err(|_| fatal("invalid_arguments", "content_base64 is invalid"))?;
    write_bytes(
        project_root,
        checkpoint_root,
        &args.path,
        &bytes,
        args.expected_sha256.as_deref(),
        1024 * 1024,
    )
}

pub(crate) fn write_bytes(
    project_root: Option<&Path>,
    checkpoint_root: Option<&Path>,
    path: &str,
    bytes: &[u8],
    expected_sha256: Option<&str>,
    max_bytes: usize,
) -> Result<Value, BusinessError> {
    let _guard = MUTATIONS
        .lock()
        .map_err(|_| fatal("write_failed", "mutation coordinator is unavailable"))?;
    let root = project_root.ok_or(fatal(
        "project_unconfigured",
        "project root is not configured",
    ))?;
    if bytes.len() > max_bytes {
        return Err(fatal(
            "output_limit",
            "file content exceeds the write limit",
        ));
    }
    let (relative, candidate, current) = target(root, path)?;
    let existed = current.is_some();
    match (current.as_deref(), expected_sha256) {
        (Some(actual), Some(expected)) => check_unchanged(actual, expected)?,
        (Some(_), None) => {
            return Err(fatal(
                "precondition_required",
                "expected_sha256 is required for an existing file",
            ))
        }
        (None, Some(_)) => return Err(fatal("write_conflict", "file no longer exists")),
        (None, None) => {}
    }
    if let Some(parent) = candidate.parent() {
        ensure_parent_directory(root, parent)?;
    }
    let staged = stage(&candidate, bytes, existed)?;
    let checkpoint_root = checkpoint_root.ok_or(fatal(
        "checkpoint_unavailable",
        "checkpoint storage is not configured",
    ))?;
    let day = checkpoint::today();
    let baseline_sha256 = current
        .as_deref()
        .map(|bytes| checkpoint::store(checkpoint_root, &day, bytes))
        .transpose()?;
    let current_sha256 = checkpoint::store(checkpoint_root, &day, bytes)?;
    publish(root, path, &candidate, staged, current.as_deref())?;
    // `checkpoint` is runtime metadata; the agent core records it as the
    // turn's file change and strips it before the model sees the result.
    Ok(json!({
        "path": path,
        "bytes": bytes.len(),
        "created": !existed,
        "sha256": current_sha256,
        "checkpoint": {
            "path": checkpoint::change_path(&relative),
            "stored_on": day,
            "baseline_sha256": baseline_sha256,
            "current_sha256": current_sha256,
        },
    }))
}

/// Deletes one regular project file whose bytes still match what the model
/// read, keeping its content as the turn change baseline.
pub(crate) fn delete(
    project_root: Option<&Path>,
    checkpoint_root: Option<&Path>,
    args: &DeleteArguments,
) -> Result<Value, BusinessError> {
    let _guard = MUTATIONS
        .lock()
        .map_err(|_| fatal("write_failed", "mutation coordinator is unavailable"))?;
    let root = project_root.ok_or(fatal(
        "project_unconfigured",
        "project root is not configured",
    ))?;
    let path = args.path.as_str();
    let (relative, candidate, current) = target(root, path)?;
    let current = current.ok_or(fatal("not_found", "file does not exist"))?;
    check_unchanged(&current, &args.expected_sha256)?;
    let checkpoint_root = checkpoint_root.ok_or(fatal(
        "project_unconfigured",
        "checkpoint storage is not configured",
    ))?;
    let day = checkpoint::today();
    let baseline_sha256 = checkpoint::store(checkpoint_root, &day, &current)?;
    recheck(root, path, Some(&current))?;
    fs::remove_file(&candidate)
        .map_err(|_| retryable("delete_failed", "file could not be deleted"))?;
    Ok(json!({
        "path": path,
        "bytes": current.len(),
        "deleted": true,
        "checkpoint": {
            "path": checkpoint::change_path(&relative),
            "stored_on": day,
            "baseline_sha256": baseline_sha256,
            "current_sha256": null,
        },
    }))
}

/// Resolves `path` inside the project without following a final symbolic
/// link, returning the current bytes when it is an existing regular file.
fn target(root: &Path, path: &str) -> Result<(PathBuf, PathBuf, Option<Vec<u8>>), BusinessError> {
    let relative = safe_relative_path(path)?;
    let candidate = root.join(&relative);
    let metadata = match fs::symlink_metadata(&candidate) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            let mut ancestor = candidate
                .parent()
                .ok_or_else(|| fatal("path_unavailable", "parent is unavailable"))?
                .to_path_buf();
            while !ancestor.exists() {
                if !ancestor.pop() {
                    return Err(fatal("path_unavailable", "parent is unavailable"));
                }
            }
            let canonical_parent = ancestor
                .canonicalize()
                .map_err(|_| fatal("path_unavailable", "parent is unavailable"))?;
            if !canonical_parent.starts_with(root) {
                return Err(fatal(
                    "scope_denied",
                    "parent directory is outside the project",
                ));
            }
            return Ok((relative, candidate, None));
        }
        Err(_) => return Err(fatal("path_unavailable", "path is unavailable")),
    };
    if metadata.file_type().is_symlink() {
        return Err(fatal(
            "scope_denied",
            "write through symbolic links are not allowed",
        ));
    }
    let canonical = candidate
        .canonicalize()
        .map_err(|_| fatal("path_unavailable", "path is unavailable"))?;
    if !canonical.starts_with(root) {
        return Err(fatal("scope_denied", "path is outside the project root"));
    }
    let metadata =
        fs::metadata(&canonical).map_err(|_| fatal("path_unavailable", "path is unavailable"))?;
    if !metadata.is_file() {
        return Err(fatal("not_a_file", "path is not a regular file"));
    }
    let mut current = Vec::new();
    fs::File::open(canonical)
        .and_then(|file| {
            file.take(crate::text::MAX_EDIT_BYTES as u64 + 1)
                .read_to_end(&mut current)
        })
        .map_err(|_| fatal("read_failed", "file content could not be read"))?;
    if current.len() > crate::text::MAX_EDIT_BYTES {
        return Err(fatal("resource_limit", "mutation source exceeds 16 MiB"));
    }
    Ok((relative, candidate, Some(current)))
}

fn check_unchanged(actual: &[u8], expected_sha256: &str) -> Result<(), BusinessError> {
    if expected_sha256.len() != 64 || !expected_sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(fatal("invalid_arguments", "expected_sha256 is invalid"));
    }
    if !sha256_hex(actual).eq_ignore_ascii_case(expected_sha256) {
        return Err(fatal(
            "write_conflict",
            "file content has changed since the last read",
        ));
    }
    Ok(())
}

fn stage(
    candidate: &Path,
    bytes: &[u8],
    existed: bool,
) -> Result<tempfile::NamedTempFile, BusinessError> {
    let mut staged = tempfile::Builder::new()
        .prefix(".suncode-edit-")
        .tempfile_in(
            candidate
                .parent()
                .ok_or_else(|| fatal("path_unavailable", "parent is unavailable"))?,
        )
        .map_err(|_| retryable("write_failed", "temporary file could not be created"))?;
    if existed {
        let permissions = fs::metadata(candidate)
            .map_err(|_| fatal("write_conflict", "file changed before staging"))?
            .permissions();
        if permissions.readonly() {
            return Err(fatal("write_failed", "target file is read-only"));
        }
        staged
            .as_file()
            .set_permissions(permissions)
            .map_err(|_| fatal("write_failed", "file permissions could not be preserved"))?;
        #[cfg(windows)]
        windows::copy_dacl(candidate, staged.path()).map_err(|_| {
            fatal(
                "write_failed",
                "file access permissions could not be preserved",
            )
        })?;
    }
    staged
        .write_all(bytes)
        .and_then(|()| staged.as_file().sync_all())
        .map_err(|_| {
            retryable(
                "write_failed",
                "temporary file could not be written and synced",
            )
        })?;
    Ok(staged)
}

fn recheck(root: &Path, path: &str, baseline: Option<&[u8]>) -> Result<(), BusinessError> {
    let (_, _, current) = target(root, path)?;
    match (current.as_deref(), baseline) {
        (Some(actual), Some(expected)) if actual == expected => Ok(()),
        (None, None) => Ok(()),
        _ => Err(fatal(
            "write_conflict",
            "file changed before commit; read the current file and retry the intended change",
        )),
    }
}

fn publish(
    root: &Path,
    path: &str,
    candidate: &Path,
    staged: tempfile::NamedTempFile,
    baseline: Option<&[u8]>,
) -> Result<(), BusinessError> {
    // Re-resolve containment and content after staging and checkpoint storage.
    recheck(root, path, baseline)?;
    if baseline.is_none() {
        staged.persist_noclobber(candidate).map_err(|error| {
            if error.error.kind() == ErrorKind::AlreadyExists {
                fatal("write_conflict", "file was created by another writer")
            } else {
                retryable("write_failed", "new file could not be committed")
            }
        })?;
    } else {
        replace(staged, candidate).map_err(|error| {
            fatal(
                "write_failed",
                "file could not be atomically replaced; read it before retrying",
            )
            .with_details(json!({"os_error": error.raw_os_error()}))
        })?;
    }
    Ok(())
}

#[cfg(not(windows))]
fn replace(staged: tempfile::NamedTempFile, candidate: &Path) -> std::io::Result<()> {
    staged
        .persist(candidate)
        .map(|_| ())
        .map_err(|error| error.error)
}

#[cfg(windows)]
fn replace(staged: tempfile::NamedTempFile, candidate: &Path) -> std::io::Result<()> {
    windows::replace(staged, candidate)
}

fn ensure_parent_directory(root: &Path, parent: &Path) -> Result<(), BusinessError> {
    let mut existing = parent.to_path_buf();
    while !existing.exists() {
        if !existing.pop() {
            return Err(fatal("path_unavailable", "parent directory is unavailable"));
        }
    }
    let canonical_existing = existing
        .canonicalize()
        .map_err(|_| fatal("path_unavailable", "parent directory is unavailable"))?;
    if !canonical_existing.starts_with(root) {
        return Err(fatal(
            "scope_denied",
            "parent directory is outside the project",
        ));
    }
    fs::create_dir_all(parent)
        .map_err(|_| retryable("write_failed", "parent directory could not be created"))?;
    let canonical_parent = parent
        .canonicalize()
        .map_err(|_| fatal("path_unavailable", "parent directory is unavailable"))?;
    if !canonical_parent.starts_with(root) {
        return Err(fatal(
            "scope_denied",
            "parent directory is outside the project",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readers_never_observe_partial_publication() {
        use std::sync::atomic::{AtomicBool, Ordering};
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let path = root.join("file");
        let first = vec![b'a'; 256 * 1024];
        let second = vec![b'b'; 512 * 1024];
        fs::write(&path, &first).unwrap();
        let barrier = std::sync::Barrier::new(2);
        let done = AtomicBool::new(false);
        std::thread::scope(|scope| {
            let reader = scope.spawn(|| {
                barrier.wait();
                while !done.load(Ordering::Acquire) {
                    let observed = fs::read(&path).unwrap();
                    assert!(observed == first || observed == second);
                }
            });
            barrier.wait();
            let writes = (|| -> Result<(), BusinessError> {
                for index in 0..20 {
                    let (baseline, next) = if index % 2 == 0 {
                        (&first, &second)
                    } else {
                        (&second, &first)
                    };
                    let staged = stage(&path, next, true).unwrap();
                    publish(&root, "file", &path, staged, Some(baseline))?;
                }
                Ok(())
            })();
            done.store(true, Ordering::Release);
            reader.join().unwrap();
            writes.unwrap();
        });
    }

    #[test]
    fn staged_commit_rejects_external_changes_and_racing_creators() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let path = root.join("file");
        fs::write(&path, b"before").unwrap();
        let staged = stage(&path, b"after", true).unwrap();
        fs::write(&path, b"external").unwrap();
        assert_eq!(
            publish(&root, "file", &path, staged, Some(b"before"))
                .unwrap_err()
                .code,
            "write_conflict"
        );
        assert_eq!(fs::read(&path).unwrap(), b"external");
        fs::remove_file(&path).unwrap();
        let staged = stage(&path, b"after", false).unwrap();
        fs::write(&path, b"creator").unwrap();
        assert_eq!(
            publish(&root, "file", &path, staged, None)
                .unwrap_err()
                .code,
            "write_conflict"
        );
        assert_eq!(fs::read(&path).unwrap(), b"creator");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    }

    #[test]
    fn failed_checkpoint_keeps_target_and_cleans_staged_file() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        fs::write(root.join("file"), b"before").unwrap();
        fs::write(root.join("versions"), b"not a directory").unwrap();
        let result = write_bytes(
            Some(&root),
            Some(&root.join("versions")),
            "file",
            b"after",
            Some(&sha256_hex(b"before")),
            1024,
        );
        assert!(result.is_err());
        assert_eq!(fs::read(root.join("file")).unwrap(), b"before");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
    }

    #[test]
    fn concurrent_writers_with_same_baseline_have_one_winner() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        fs::write(root.join("file"), b"before").unwrap();
        let barrier = std::sync::Barrier::new(2);
        let results = std::thread::scope(|scope| {
            let workers: Vec<_> = [b"first".as_slice(), b"second".as_slice()]
                .into_iter()
                .map(|bytes| {
                    let root = &root;
                    let barrier = &barrier;
                    scope.spawn(move || {
                        barrier.wait();
                        write_bytes(
                            Some(root),
                            Some(&root.join("versions")),
                            "file",
                            bytes,
                            Some(&sha256_hex(b"before")),
                            1024,
                        )
                    })
                })
                .collect();
            workers
                .into_iter()
                .map(|worker| worker.join().unwrap())
                .collect::<Vec<_>>()
        });
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(
            results
                .iter()
                .filter_map(|result| result.as_ref().err())
                .next()
                .unwrap()
                .code,
            "write_conflict"
        );
        assert!(matches!(
            fs::read(root.join("file")).unwrap().as_slice(),
            b"first" | b"second"
        ));
    }

    #[test]
    fn successful_replace_keeps_permissions_and_cleans_staging() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let path = root.join("file");
        fs::write(&path, b"before").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o751)).unwrap();
        }
        write_bytes(
            Some(&root),
            Some(&root.join("versions")),
            "file",
            b"after",
            Some(&sha256_hex(b"before")),
            1024,
        )
        .unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"after");
        assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o751
            );
        }
    }
}
