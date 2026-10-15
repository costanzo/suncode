use std::collections::BTreeMap;

pub(super) fn default_environment(
    working_directory: &std::path::Path,
    configured: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    // GUI-launched hosts often have a sparse environment; rebuild only the
    // launch/runtime values MCP processes need instead of inheriting secrets.
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
            "HOMEDRIVE",
            "HOMEPATH",
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
            .or_insert_with(|| system_root.clone());
        values
            .entry("WINDIR".into())
            .or_insert_with(|| system_root.clone());
        values
            .entry("OS".into())
            .or_insert_with(|| "Windows_NT".into());
        values
            .entry("ComSpec".into())
            .or_insert_with(|| format!("{system_root}\\System32\\cmd.exe"));
        values
            .entry("PATHEXT".into())
            .or_insert_with(|| ".COM;.EXE;.BAT;.CMD;.VBS;.VBE;.JS;.JSE;.WSF;.WSH;.MSC".into());

        if let Some(user_profile) = values.get("USERPROFILE").cloned() {
            values
                .entry("HOME".into())
                .or_insert_with(|| user_profile.clone());
            values
                .entry("APPDATA".into())
                .or_insert_with(|| format!("{user_profile}\\AppData\\Roaming"));
            values
                .entry("LOCALAPPDATA".into())
                .or_insert_with(|| format!("{user_profile}\\AppData\\Local"));
        }
        let temp = host("TEMP")
            .or_else(|| host("TMP"))
            .unwrap_or_else(|| std::env::temp_dir().to_string_lossy().into_owned());
        values.entry("TEMP".into()).or_insert_with(|| temp.clone());
        values.entry("TMP".into()).or_insert(temp);

        let path = host("Path").or_else(|| host("PATH")).unwrap_or_else(|| {
            format!("{system_root}\\System32;{system_root};{system_root}\\System32\\Wbem")
        });
        values.entry("PATH".into()).or_insert(path);
    }

    #[cfg(not(windows))]
    {
        for key in [
            "HOME",
            "USER",
            "LOGNAME",
            "SHELL",
            "SSH_AUTH_SOCK",
            "GPG_AGENT_INFO",
            "DISPLAY",
            "WAYLAND_DISPLAY",
            "DBUS_SESSION_BUS_ADDRESS",
            "TERM_PROGRAM",
            "COLORTERM",
            "NVM_DIR",
            "NVM_BIN",
            "VOLTA_HOME",
            "PNPM_HOME",
            "ASDF_DATA_DIR",
            "MISE_DATA_DIR",
            "VIRTUAL_ENV",
            "CONDA_PREFIX",
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
            "__CF_USER_TEXT_ENCODING",
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
            "/snap/bin".to_string(),
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
        for key in ["NVM_BIN", "PNPM_HOME"] {
            if let Some(value) = values.get(key) {
                paths.insert(0, value.clone());
            }
        }
        for (key, suffix) in [
            ("VOLTA_HOME", "bin"),
            ("ASDF_DATA_DIR", "shims"),
            ("MISE_DATA_DIR", "shims"),
            ("VIRTUAL_ENV", "bin"),
            ("CONDA_PREFIX", "bin"),
        ] {
            if let Some(value) = values.get(key) {
                paths.insert(0, format!("{value}/{suffix}"));
            }
        }
        if let Some(existing) = host("PATH") {
            paths.push(existing);
        }
        let mut deduplicated_paths = Vec::with_capacity(paths.len());
        for path in paths {
            if !deduplicated_paths.iter().any(|value| value == &path) {
                deduplicated_paths.push(path);
            }
        }
        let path = deduplicated_paths.join(":");
        values.insert("PATH".into(), path);
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
        if let Some(home) = home {
            values
                .entry("XDG_CONFIG_HOME".into())
                .or_insert_with(|| format!("{home}/.config"));
            values
                .entry("XDG_DATA_HOME".into())
                .or_insert_with(|| format!("{home}/.local/share"));
            values
                .entry("XDG_CACHE_HOME".into())
                .or_insert_with(|| format!("{home}/.cache"));
        }
        values.insert(
            "PWD".into(),
            working_directory.to_string_lossy().into_owned(),
        );
    }

    for (key, value) in configured {
        #[cfg(windows)]
        {
            let existing = values
                .keys()
                .find(|existing| existing.eq_ignore_ascii_case(key))
                .cloned();
            if let Some(existing) = existing {
                values.remove(&existing);
            }
        }
        values.insert(key.clone(), value.clone());
    }
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_environment_includes_platform_baseline_and_working_directory() {
        let working_directory = std::path::Path::new("/tmp/suncode-mcp-project");
        let values = default_environment(working_directory, &BTreeMap::new());
        assert!(values.get("PATH").is_some_and(|value| !value.is_empty()));
        assert!(
            values.get("TMPDIR").is_some_and(|value| !value.is_empty())
                || values.get("TEMP").is_some_and(|value| !value.is_empty())
        );
        #[cfg(not(windows))]
        {
            // Locale is inherited from the host when set, otherwise defaults to C.UTF-8.
            for key in ["LANG", "LC_ALL"] {
                let expected = std::env::var(key)
                    .ok()
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "C.UTF-8".into());
                assert_eq!(values.get(key), Some(&expected), "{key}");
            }
            assert_eq!(
                values.get("PWD").map(String::as_str),
                Some("/tmp/suncode-mcp-project")
            );
        }
        #[cfg(windows)]
        {
            assert!(values.contains_key("SystemRoot"));
            assert!(values.contains_key("ComSpec"));
        }
    }

    #[test]
    fn configured_environment_overrides_defaults_without_removing_baseline() {
        let configured = BTreeMap::from([
            ("PATH".into(), "/custom/bin".into()),
            ("MCP_TOKEN".into(), "secret".into()),
        ]);
        let values = default_environment(std::path::Path::new("/tmp/project"), &configured);
        assert_eq!(values.get("PATH").map(String::as_str), Some("/custom/bin"));
        assert_eq!(values.get("MCP_TOKEN").map(String::as_str), Some("secret"));
        assert!(values.contains_key("TMPDIR") || values.contains_key("TEMP"));
    }
}
