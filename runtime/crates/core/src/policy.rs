#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Risk {
    ReadOnly,
    ProjectWrite,
    ProcessExecution,
    DestructiveWrite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Allow,
    ApprovalRequired,
    Deny,
}

pub fn evaluate(risk: Option<Risk>, non_interactive: bool, full_control: bool) -> Decision {
    match risk {
        None => Decision::Deny,
        Some(Risk::ReadOnly) => Decision::Allow,
        Some(_) if full_control => Decision::Allow,
        Some(_) if non_interactive => Decision::Deny,
        Some(_) => Decision::ApprovalRequired,
    }
}

pub fn tool_risk(name: &str) -> Option<Risk> {
    match name {
        "read" | "glob" | "grep" | "project_inspect" | "project.inspect" | "fs_read"
        | "fs.read" | "fs_metadata" | "fs.metadata" | "search_glob" | "search.glob"
        | "search_find" | "search.find" => Some(Risk::ReadOnly),
        "write" | "edit" | "apply_patch" | "fs_write" | "fs.write" | "fs_edit" | "fs.edit"
        | "fs_patch" | "fs.patch" | "fs_move" | "fs.move" => Some(Risk::ProjectWrite),
        "fs_delete" | "fs.delete" => Some(Risk::DestructiveWrite),
        "bash" | "shell" | "shell_run" | "shell.run" | "process" | "process_run"
        | "process.run" => Some(Risk::ProcessExecution),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_match_contract() {
        assert_eq!(evaluate(tool_risk("read"), false, false), Decision::Allow);
        assert_eq!(
            evaluate(tool_risk("write"), false, false),
            Decision::ApprovalRequired
        );
        assert_eq!(
            evaluate(tool_risk("bash"), false, false),
            Decision::ApprovalRequired
        );
        assert_eq!(
            evaluate(tool_risk("shell"), false, false),
            Decision::ApprovalRequired
        );
        assert_eq!(
            evaluate(tool_risk("process"), false, false),
            Decision::ApprovalRequired
        );
        assert_eq!(evaluate(tool_risk("write"), true, false), Decision::Deny);
        assert_eq!(evaluate(tool_risk("unknown"), false, false), Decision::Deny);
    }

    #[test]
    fn full_control_allows_known_risks_but_not_unknown_tools() {
        assert_eq!(evaluate(tool_risk("write"), false, true), Decision::Allow);
        assert_eq!(evaluate(tool_risk("bash"), false, true), Decision::Allow);
        assert_eq!(
            evaluate(tool_risk("fs_delete"), false, true),
            Decision::Allow
        );
        assert_eq!(evaluate(tool_risk("unknown"), false, true), Decision::Deny);
    }
}
