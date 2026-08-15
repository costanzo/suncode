use fs2::FileExt;
use std::{
    fs::{self, File, OpenOptions},
    io,
    path::Path,
};

#[derive(Debug)]
pub struct RuntimeLock {
    lock: File,
}

impl RuntimeLock {
    pub fn acquire(data_dir: &Path) -> io::Result<Self> {
        fs::create_dir_all(data_dir)?;
        let lock_path = data_dir.join("runtime.lock");
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&lock_path)?;
        restrict_permissions(&lock_path)?;
        lock.try_lock_exclusive().map_err(|error| {
            if error.kind() == io::ErrorKind::WouldBlock {
                io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "another SunCode runtime is already active",
                )
            } else {
                error
            }
        })?;
        Ok(Self { lock })
    }
}

impl Drop for RuntimeLock {
    fn drop(&mut self) {
        let _ = self.lock.unlock();
    }
}

#[cfg(unix)]
fn restrict_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prevents_a_second_runtime_without_publishing_an_endpoint() {
        let directory = tempfile::tempdir().unwrap();
        let _first = RuntimeLock::acquire(directory.path()).unwrap();
        assert_eq!(
            RuntimeLock::acquire(directory.path()).unwrap_err().kind(),
            io::ErrorKind::AlreadyExists
        );
        assert!(!directory.path().join("runtime.json").exists());
    }
}
