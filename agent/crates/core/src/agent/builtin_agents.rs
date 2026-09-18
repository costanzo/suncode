//! Immutable specialist-agent catalog compiled into the Rust agent.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuiltinAgentDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub version: i64,
    pub instructions: &'static str,
    pub allowed_tools: &'static [&'static str],
    pub model_policy: &'static str,
    pub mcp_policy: &'static str,
    pub can_delegate: bool,
    pub tool_call_limit: u32,
}

const READ_TOOLS: &[&str] = &["read", "glob", "grep", "webfetch", "todowrite"];
const IMPLEMENTATION_TOOLS: &[&str] = &[
    "read",
    "glob",
    "grep",
    "write",
    "edit",
    "bash",
    "todowrite",
    "webfetch",
];

static CATALOG: &[BuiltinAgentDefinition] = &[
    BuiltinAgentDefinition {
        id: "builtin.architect.v1",
        name: "architect-agent",
        display_name: "Architect",
        description: "Owns architecture analysis, boundaries, contracts, and technical tradeoffs.",
        version: 1,
        instructions: "Act as SunCode's architecture specialist. Explain boundaries, dependencies, invariants, migration risks, and a concrete implementation path. Do not edit files or run processes.",
        allowed_tools: READ_TOOLS,
        model_policy: "Inherits parent session model",
        mcp_policy: "Not allowed",
        can_delegate: false,
        tool_call_limit: 32,
    },
    BuiltinAgentDefinition {
        id: "builtin.ui-ux.v1",
        name: "ui-ux-agent",
        display_name: "UI/UX Agent",
        description: "Reviews interfaces, interaction states, accessibility, and design-system alignment.",
        version: 1,
        instructions: "Act as SunCode's UI/UX specialist. Inspect the existing design system and product code, then provide specific hierarchy, interaction, accessibility, and responsive recommendations. Do not edit files or run processes.",
        allowed_tools: READ_TOOLS,
        model_policy: "Inherits parent session model",
        mcp_policy: "Not allowed",
        can_delegate: false,
        tool_call_limit: 32,
    },
    BuiltinAgentDefinition {
        id: "builtin.product.v1",
        name: "product-agent",
        display_name: "Product Agent",
        description: "Clarifies requirements, acceptance criteria, user flows, and edge cases.",
        version: 1,
        instructions: "Act as SunCode's product specialist. Turn ambiguous requests into explicit goals, scope, acceptance criteria, edge cases, and user-visible behavior. Do not edit files or run processes.",
        allowed_tools: READ_TOOLS,
        model_policy: "Inherits parent session model",
        mcp_policy: "Not allowed",
        can_delegate: false,
        tool_call_limit: 32,
    },
    BuiltinAgentDefinition {
        id: "builtin.swe.v1",
        name: "swe-agent",
        display_name: "Software Engineering Agent",
        description: "Implements focused software changes and verifies the affected code paths.",
        version: 1,
        instructions: "Act as SunCode's software engineer. Implement the delegated change with the smallest coherent diff, preserve project boundaries, add focused tests, and report verification and residual risk.",
        allowed_tools: IMPLEMENTATION_TOOLS,
        model_policy: "Inherits parent session model",
        mcp_policy: "Not allowed",
        can_delegate: false,
        tool_call_limit: 64,
    },
    BuiltinAgentDefinition {
        id: "builtin.qa.v1",
        name: "qa-agent",
        display_name: "QA Agent",
        description: "Designs and runs focused tests, regression checks, and failure analysis.",
        version: 1,
        instructions: "Act as SunCode's QA specialist. Build a focused verification plan, run relevant tests and checks, reproduce failures, and report exact evidence. Make test-only changes when needed; do not make unrelated product changes.",
        allowed_tools: IMPLEMENTATION_TOOLS,
        model_policy: "Inherits parent session model",
        mcp_policy: "Not allowed",
        can_delegate: false,
        tool_call_limit: 64,
    },
    BuiltinAgentDefinition {
        id: "builtin.sre.v1",
        name: "sre-agent",
        display_name: "SRE Agent",
        description: "Handles operations, observability, recovery, performance, and runtime reliability.",
        version: 1,
        instructions: "Act as SunCode's SRE specialist. Inspect operational behavior, logs, monitoring, recovery, performance, and deployment-adjacent risks. Make narrowly scoped operational changes and verify them honestly.",
        allowed_tools: IMPLEMENTATION_TOOLS,
        model_policy: "Inherits parent session model",
        mcp_policy: "Not allowed",
        can_delegate: false,
        tool_call_limit: 64,
    },
];

pub fn all() -> &'static [BuiltinAgentDefinition] {
    CATALOG
}

pub fn by_name(name: &str) -> Option<&'static BuiltinAgentDefinition> {
    CATALOG.iter().find(|agent| agent.name == name)
}

pub fn by_id(id: &str) -> Option<&'static BuiltinAgentDefinition> {
    CATALOG.iter().find(|agent| agent.id == id)
}

#[cfg(test)]
mod tests {
    use super::{all, IMPLEMENTATION_TOOLS, READ_TOOLS};
    use std::collections::BTreeSet;

    #[test]
    fn catalog_has_six_unique_fixed_agents() {
        let names = all()
            .iter()
            .map(|agent| agent.name)
            .collect::<BTreeSet<_>>();
        let ids = all().iter().map(|agent| agent.id).collect::<BTreeSet<_>>();
        assert_eq!(all().len(), 6);
        assert_eq!(names.len(), all().len());
        assert_eq!(ids.len(), all().len());
        assert!(all().iter().all(|agent| !agent.can_delegate));
        assert!(all()
            .iter()
            .all(|agent| !agent.allowed_tools.contains(&"question")));
    }

    #[test]
    fn catalog_assigns_the_exact_role_tool_allowlists_and_limits() {
        for agent in all() {
            match agent.name {
                "architect-agent" | "ui-ux-agent" | "product-agent" => {
                    assert_eq!(agent.allowed_tools, READ_TOOLS);
                    assert_eq!(agent.tool_call_limit, 32);
                }
                "swe-agent" | "qa-agent" | "sre-agent" => {
                    assert_eq!(agent.allowed_tools, IMPLEMENTATION_TOOLS);
                    assert_eq!(agent.tool_call_limit, 64);
                }
                name => panic!("unexpected built-in agent {name}"),
            }
            assert!(!agent
                .allowed_tools
                .iter()
                .any(|tool| tool.starts_with("mcp_")));
            assert!(!agent.allowed_tools.contains(&"delegate_agent"));
        }
    }
}
