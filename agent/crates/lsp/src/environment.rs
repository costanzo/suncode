//! Minimal, explicit process environment for spawned language servers.

use std::{collections::BTreeMap, path::Path};

pub(crate) fn default_environment(
    working_directory: &Path,
    configured: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    let host = |key: &str| std::env::var(key).ok().filter(|value| !value.is_empty());
    let insert_host = |values: &mut BTreeMap<String, String>, key: &str| {
        if let Some(value) = host(key) {
            values.insert(key.to_string(), value);
        }
    };

    #[cfg(windows)]
    {
        for key in [
            "SystemRoot",
            "WINDIR",
            "USERPROFILE",
            "APPDATA",
            "LOCALAPPDATA",
            "ComSpec",
            "PATHEXT",
            "USERNAME",
            "OS",
            "http_proxy",
            "https_proxy",
            "no_proxy",
            "HTTP_PROXY",
            "HTTPS_PROXY",
            "NO_PROXY",
        ] {
            insert_host(&mut values, key);
        }
        let system_root = values
            .get("SystemRoot")
            .or_else(|| values.get("WINDIR"))
            .cloned()
            .unwrap_or_else(|| r"C:\Windows".to_string());
        values
            .entry("SystemRoot".into())
            .or_insert(system_root.clone());
        values.entry("WINDIR".into()).or_insert(system_root.clone());
        values
            .entry("ComSpec".into())
            .or_insert_with(|| format!(r"{system_root}\System32\cmd.exe"));
        values
            .entry("PATHEXT".into())
            .or_insert_with(|| ".COM;.EXE;.BAT;.CMD".into());
        let temp = host("TEMP")
            .or_else(|| host("TMP"))
            .unwrap_or_else(|| std::env::temp_dir().to_string_lossy().into_owned());
        values.entry("TEMP".into()).or_insert(temp.clone());
        values.entry("TMP".into()).or_insert(temp);
        values.entry("PATH".into()).or_insert_with(|| {
            host("Path")
                .or_else(|| host("PATH"))
                .unwrap_or_else(|| format!(r"{system_root}\System32;{system_root}"))
        });
    }

    #[cfg(not(windows))]
    {
        for key in [
            "HOME",
            "USER",
            "LOGNAME",
            "SHELL",
            "SSH_AUTH_SOCK",
            "LANG",
            "LC_ALL",
            "XDG_CONFIG_HOME",
            "XDG_DATA_HOME",
            "XDG_CACHE_HOME",
            "http_proxy",
            "https_proxy",
            "no_proxy",
            "HTTP_PROXY",
            "HTTPS_PROXY",
            "NO_PROXY",
            "NVM_BIN",
            "PNPM_HOME",
            "VIRTUAL_ENV",
            "CONDA_PREFIX",
        ] {
            insert_host(&mut values, key);
        }
        let home = values.get("HOME").cloned();
        let mut paths = vec![
            "/opt/homebrew/bin".to_string(),
            "/usr/local/bin".to_string(),
            "/usr/bin".to_string(),
            "/bin".to_string(),
            "/usr/sbin".to_string(),
            "/sbin".to_string(),
        ];
        if let Some(home) = &home {
            paths.splice(
                0..0,
                [
                    format!("{home}/.local/bin"),
                    format!("{home}/.cargo/bin"),
                    format!("{home}/bin"),
                ],
            );
        }
        if let Some(existing) = host("PATH") {
            paths.push(existing);
        }
        paths.dedup();
        values.insert("PATH".into(), paths.join(":"));
        values
            .entry("TMPDIR".into())
            .or_insert_with(|| std::env::temp_dir().to_string_lossy().into_owned());
        values
            .entry("LANG".into())
            .or_insert_with(|| "C.UTF-8".into());
        values
            .entry("LC_ALL".into())
            .or_insert_with(|| "C.UTF-8".into());
        values
            .entry("SHELL".into())
            .or_insert_with(|| "/bin/sh".into());
        values.entry("TERM".into()).or_insert_with(|| "dumb".into());
        values.insert(
            "PWD".into(),
            working_directory.to_string_lossy().into_owned(),
        );
    }

    for (key, value) in configured {
        values.insert(key.clone(), value.clone());
    }
    values
}
