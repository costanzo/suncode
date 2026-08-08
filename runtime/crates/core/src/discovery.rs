use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscoveryRecord {
    pub endpoint: String,
    pub token: String,
    pub runtime_version: String,
    pub protocol_version: u32,
    pub process_id: u32,
}

#[derive(Debug)]
pub struct RuntimeDiscovery {
    lock: File,
    path: PathBuf,
    token: String,
}

impl RuntimeDiscovery {
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
                    "another Suncode runtime is already active",
                )
            } else {
                error
            }
        })?;
        Ok(Self {
            lock,
            path: data_dir.join("runtime.json"),
            token: String::new(),
        })
    }

    pub fn publish(&mut self, endpoint: String, token: String) -> io::Result<()> {
        let record = DiscoveryRecord {
            endpoint,
            token: token.clone(),
            runtime_version: env!("CARGO_PKG_VERSION").to_string(),
            protocol_version: 1,
            process_id: std::process::id(),
        };
        let temporary = self
            .path
            .with_extension(format!("json.{}.tmp", std::process::id()));
        fs::write(
            &temporary,
            serde_json::to_vec(&record).map_err(io::Error::other)?,
        )?;
        restrict_permissions(&temporary)?;
        fs::rename(&temporary, &self.path)?;
        self.token = token;
        Ok(())
    }
}

impl Drop for RuntimeDiscovery {
    fn drop(&mut self) {
        let owned = fs::read(&self.path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<DiscoveryRecord>(&bytes).ok())
            .is_some_and(|record| record.token == self.token);
        if owned {
            let _ = fs::remove_file(&self.path);
        }
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
    fn publishes_restricted_record_and_prevents_second_runtime() {
        let directory = tempfile::tempdir().unwrap();
        let mut first = RuntimeDiscovery::acquire(directory.path()).unwrap();
        first
            .publish("http://127.0.0.1:1234".into(), "secret".into())
            .unwrap();
        let record: DiscoveryRecord =
            serde_json::from_slice(&fs::read(directory.path().join("runtime.json")).unwrap())
                .unwrap();
        assert_eq!(record.endpoint, "http://127.0.0.1:1234");
        assert_eq!(record.protocol_version, 1);
        assert_eq!(
            RuntimeDiscovery::acquire(directory.path())
                .unwrap_err()
                .kind(),
            io::ErrorKind::AlreadyExists
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(directory.path().join("runtime.json"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
        drop(first);
        assert!(!directory.path().join("runtime.json").exists());
    }
}
