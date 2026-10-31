//! Bash argument validation, the hard-deny filter, and host-supplied shell execution.

use serde_json::{json, Value};
use suncode_common::BusinessError;

pub(crate) fn validate_bash_arguments(value: &Value) -> Result<(), BusinessError> {
    let command = value
        .get("command")
        .and_then(Value::as_str)
        .filter(|command| !command.trim().is_empty())
        .ok_or_else(|| {
            BusinessError::new(
                "invalid_arguments",
                "bash command must be a non-empty string",
            )
        })?;
    let normalized = normalize_shell_command(command);

    let blocked = if contains_root_delete(&normalized) {
        Some("root_or_home_recursive_delete")
    } else if contains_any(
        &normalized,
        &["find / -delete", "find / -exec rm", "find / -exec shred"],
    ) {
        Some("root_filesystem_delete")
    } else if contains_any(
        &normalized,
        &[
            "of=/dev/",
            "of=/dev/disk",
            "mkfs.",
            "mkfs ",
            "wipefs ",
            "shred /dev/",
            "diskutil erasedisk",
            "format-volume",
            "clear-disk",
            "diskpart",
        ],
    ) {
        Some("disk_or_partition_destruction")
    } else if contains_any(
        &normalized,
        &[
            "curl ",
            "wget ",
            "iwr ",
            "irm ",
            "invoke-webrequest",
            "invoke-restmethod",
        ],
    ) && contains_any(
        &normalized,
        &[
            "| sh",
            "| bash",
            "| zsh",
            "| iex",
            "| invoke-expression",
            "eval ",
            "source <",
        ],
    ) {
        Some("download_and_execute")
    } else if contains_any(
        &normalized,
        &[
            "powershell -enc",
            "powershell -encodedcommand",
            "powershell -e ",
            "pwsh -enc",
            "pwsh -encodedcommand",
            "pwsh -e ",
            "invoke-expression ",
            "set-executionpolicy bypass",
        ],
    ) {
        Some("encoded_or_policy_bypassing_execution")
    } else if contains_any(
        &normalized,
        &[
            "kill -9 -1",
            "killall ",
            "pkill -9",
            "systemctl poweroff",
            "systemctl reboot",
            "shutdown -",
            "reboot",
            "halt",
            "stop-computer",
            "restart-computer",
            "taskkill /f /im *",
        ],
    ) {
        Some("system_wide_process_or_power_control")
    } else if contains_any(
        &normalized,
        &[":(){ :|:& };:", ":(){:|:&};:", "yes >", "yes /dev/null"],
    ) {
        Some("resource_exhaustion")
    } else if contains_any(
        &normalized,
        &[
            "cat ~/.ssh/id_",
            "cat $home/.ssh/id_",
            "cat ~/.aws/credentials",
            "cat $home/.aws/credentials",
            "cat /etc/shadow",
            "get-content ~/.ssh/",
            "get-content $home/.ssh/",
            "reg save hklm\\sam",
            "security find-generic-password",
        ],
    ) {
        Some("credential_or_secret_access")
    } else if contains_any(
        &normalized,
        &[
            "curl -d @",
            "curl --data @",
            "curl --data-binary @",
            "nc <",
            "ncat <",
            "invoke-restmethod -body (get-content",
        ],
    ) {
        Some("credential_or_file_exfiltration")
    } else if contains_any(
        &normalized,
        &["sudo ", "su -", "doas ", "pkexec ", "runas "],
    ) || (normalized.contains("start-process ") && normalized.contains("-verb runas"))
    {
        Some("privilege_escalation")
    } else if contains_any(
        &normalized,
        &[
            "> /etc/",
            "> /boot/",
            ">/etc/",
            ">/boot/",
            "> ~/.ssh/authorized_keys",
            ">~/.ssh/authorized_keys",
            "> $home/.ssh/authorized_keys",
            ">$home/.ssh/authorized_keys",
            "chmod -r 777 /",
            "chown -r /",
            "chown -r /etc/",
            "chown -r /boot/",
        ],
    ) {
        Some("system_security_or_startup_modification")
    } else if contains_any(
        &normalized,
        &[
            "terraform destroy",
            "aws s3 rm --recursive",
            "docker system prune -af",
            "kubectl delete namespace",
            "git push --mirror",
            "git push -f",
            "git push --force",
        ],
    ) {
        Some("irreversible_external_or_repository_destruction")
    } else {
        None
    };

    if let Some(rule) = blocked {
        return Err(BusinessError::new(
            "unsafe_command_blocked",
            "The shell command matches a permanently blocked safety rule",
        )
        .details(json!({"rule": rule})));
    }
    Ok(())
}

fn normalize_shell_command(command: &str) -> String {
    let mut normalized = String::with_capacity(command.len());
    let mut quote = None;
    let mut escaped = false;
    let mut whitespace = false;
    for character in command.chars() {
        if escaped {
            normalized.push(character.to_ascii_lowercase());
            escaped = false;
            whitespace = false;
            continue;
        }
        if character == '\\' && quote != Some('\'') {
            escaped = true;
            continue;
        }
        if let Some(active_quote) = quote {
            if character == active_quote {
                quote = None;
            } else {
                normalized.push(character.to_ascii_lowercase());
            }
            whitespace = false;
            continue;
        }
        if character == '\'' || character == '"' {
            quote = Some(character);
            whitespace = false;
        } else if character.is_whitespace() {
            if !whitespace {
                normalized.push(' ');
                whitespace = true;
            }
        } else {
            normalized.push(character.to_ascii_lowercase());
            whitespace = false;
        }
    }
    normalized
        .trim()
        .replace("${ifs}", " ")
        .replace("$ifs", " ")
}

fn contains_root_delete(command: &str) -> bool {
    [
        "rm -rf /",
        "rm -fr /",
        "rm -rf --no-preserve-root /",
        "rm -fr --no-preserve-root /",
        "rm --no-preserve-root --force --recursive /",
        "rm --no-preserve-root --recursive --force /",
        "rm --force --recursive /",
        "rm --recursive --force /",
        "remove-item / -recurse -force",
        "remove-item c:\\ -recurse -force",
        "remove-item c:\\ -force -recurse",
        "remove-item -recurse -force c:\\",
        "remove-item -force -recurse c:\\",
        "remove-item -recurse -force /",
        "remove-item -force -recurse /",
    ]
    .iter()
    .any(|prefix| command_boundary_after(command, prefix))
        || ["rm -rf ~", "rm -fr ~", "rm -rf $home", "rm -fr $home"]
            .iter()
            .any(|prefix| command.contains(prefix))
}

fn command_boundary_after(command: &str, prefix: &str) -> bool {
    command.match_indices(prefix).any(|(index, _)| {
        command[index + prefix.len()..]
            .chars()
            .next()
            .map(|character| character.is_whitespace() || ";|&>*?[.\n".contains(character))
            .unwrap_or(true)
    })
}

fn contains_any(command: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| command.contains(needle))
}

pub(crate) fn shell_command(
    script: &str,
    captured: Option<&suncode_common::shell::ResolvedShell>,
) -> Result<(String, Vec<String>), BusinessError> {
    let shell = captured.ok_or_else(|| {
        BusinessError::new(
            "shell_context_unavailable",
            "The host has not configured an agent shell",
        )
    })?;
    Ok((shell.path.clone(), command_args(shell, script)))
}

fn command_args(shell: &suncode_common::shell::ResolvedShell, script: &str) -> Vec<String> {
    if shell.family == "sh"
        && std::path::Path::new(&shell.path)
            .file_name()
            .is_some_and(|name| name == "busybox")
    {
        return vec!["sh".into(), "-c".into(), script.into()];
    }
    if shell.family == "powershell" {
        vec!["-NoLogo".into(), "-NoProfile".into(), "-NonInteractive".into(), "-Command".into(), format!("[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false); $OutputEncoding = [Console]::OutputEncoding; {script}")]
    } else if shell.family == "zsh" {
        vec!["-f".into(), "-c".into(), script.into()]
    } else {
        vec!["-c".into(), script.into()]
    }
}
