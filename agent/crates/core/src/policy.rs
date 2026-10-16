pub use suncode_common::tool::Risk;
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
        Some(Risk::ComputerObserve) => Decision::Allow,
        Some(Risk::ComputerInput) if non_interactive => Decision::Deny,
        Some(Risk::ComputerInput) => Decision::ApprovalRequired,
        Some(Risk::BrowserAccess) if non_interactive => Decision::Deny,
        Some(Risk::BrowserAccess) => Decision::ApprovalRequired,
        Some(_) if full_control => Decision::Allow,
        Some(_) if non_interactive => Decision::Deny,
        Some(_) => Decision::ApprovalRequired,
    }
}

/// Risk of tools the core runs itself: MCP and turn-level tools. Tool
/// providers classify their own tools.
pub fn tool_risk(name: &str) -> Option<Risk> {
    if name.starts_with("mcp__") {
        return Some(Risk::ExternalTool);
    }
    match name {
        "lsp_diagnostics" | "lsp_definition" | "lsp_references" | "lsp_hover" | "lsp_symbols"
        | "question" | "skill" | "todowrite" => Some(Risk::ReadOnly),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_contract() {
        assert_eq!(
            evaluate(Some(Risk::ReadOnly), false, false),
            Decision::Allow
        );
        assert_eq!(evaluate(tool_risk("skill"), false, false), Decision::Allow);
        for risk in [
            Risk::ProjectWrite,
            Risk::ProcessExecution,
            Risk::NetworkAccess,
        ] {
            assert_eq!(
                evaluate(Some(risk), false, false),
                Decision::ApprovalRequired
            );
            assert_eq!(evaluate(Some(risk), true, false), Decision::Deny);
        }
        assert_eq!(evaluate(tool_risk("unknown"), false, false), Decision::Deny);
        assert_eq!(
            evaluate(tool_risk("mcp__github__search"), false, false),
            Decision::ApprovalRequired
        );
        assert_eq!(
            evaluate(tool_risk("mcp__github__search"), true, false),
            Decision::Deny
        );
        assert_eq!(
            evaluate(Some(Risk::BrowserAccess), false, false),
            Decision::ApprovalRequired
        );
        assert_eq!(
            evaluate(Some(Risk::BrowserAccess), true, false),
            Decision::Deny
        );
    }

    #[test]
    fn full_control_allows_known_risks_but_not_unknown_tools() {
        for risk in [
            Risk::ProjectWrite,
            Risk::ProcessExecution,
            Risk::NetworkAccess,
        ] {
            assert_eq!(evaluate(Some(risk), false, true), Decision::Allow);
        }
        assert_eq!(
            evaluate(Some(Risk::BrowserAccess), false, true),
            Decision::ApprovalRequired
        );
        assert_eq!(evaluate(tool_risk("unknown"), false, true), Decision::Deny);
    }
}
