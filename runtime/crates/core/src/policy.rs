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

pub fn evaluate(risk: Option<Risk>, non_interactive: bool) -> Decision {
    match risk {
        None => Decision::Deny,
        Some(Risk::ReadOnly) => Decision::Allow,
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
        "bash" | "process_run" | "process.run" => Some(Risk::ProcessExecution),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_match_contract() {
        assert_eq!(evaluate(tool_risk("read"), false), Decision::Allow);
        assert_eq!(
            evaluate(tool_risk("write"), false),
            Decision::ApprovalRequired
        );
        assert_eq!(
            evaluate(tool_risk("bash"), false),
            Decision::ApprovalRequired
        );
        assert_eq!(evaluate(tool_risk("write"), true), Decision::Deny);
        assert_eq!(evaluate(tool_risk("unknown"), false), Decision::Deny);
    }
}
