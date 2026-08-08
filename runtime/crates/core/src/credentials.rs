use std::{
    process::{Command, Stdio},
    sync::{Arc, RwLock},
};

const SERVICE: &str = "com.suncode.runtime";

#[derive(Clone)]
pub struct CredentialStore {
    deepseek: Arc<RwLock<Option<String>>>,
}

impl CredentialStore {
    pub fn load(non_interactive: bool) -> Self {
        let value = if non_interactive {
            std::env::var("DEEPSEEK_API_KEY")
                .ok()
                .filter(|value| !value.trim().is_empty())
        } else {
            keychain_get("deepseek")
        };
        Self {
            deepseek: Arc::new(RwLock::new(value)),
        }
    }

    #[cfg(test)]
    pub fn memory(value: Option<&str>) -> Self {
        Self {
            deepseek: Arc::new(RwLock::new(value.map(str::to_string))),
        }
    }

    pub fn configured(&self) -> bool {
        self.deepseek.read().is_ok_and(|value| value.is_some())
    }

    pub fn api_key(&self) -> Option<String> {
        self.deepseek.read().ok()?.clone()
    }

    pub fn set(&self, value: &str) -> Result<(), String> {
        let value = value.trim();
        if value.is_empty() {
            return Err("credential must not be empty".into());
        }
        if cfg!(target_os = "macos") {
            let status = Command::new("/usr/bin/security")
                .args([
                    "add-generic-password",
                    "-a",
                    "deepseek",
                    "-s",
                    SERVICE,
                    "-U",
                    "-w",
                    value,
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map_err(|_| "OS credential store is unavailable")?;
            if !status.success() {
                return Err("OS credential store rejected the credential".into());
            }
            if keychain_get("deepseek").as_deref() != Some(value) {
                return Err("OS credential store did not persist the credential".into());
            }
        } else {
            return Err("OS credential store is unavailable on this platform".into());
        }
        *self
            .deepseek
            .write()
            .map_err(|_| "credential state is unavailable")? = Some(value.to_string());
        Ok(())
    }

    pub fn delete(&self) -> Result<(), String> {
        if cfg!(target_os = "macos") {
            let _ = Command::new("/usr/bin/security")
                .args(["delete-generic-password", "-a", "deepseek", "-s", SERVICE])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        *self
            .deepseek
            .write()
            .map_err(|_| "credential state is unavailable")? = None;
        Ok(())
    }
}

fn keychain_get(account: &str) -> Option<String> {
    if !cfg!(target_os = "macos") {
        return None;
    }
    let output = Command::new("/usr/bin/security")
        .args(["find-generic-password", "-a", account, "-s", SERVICE, "-w"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
