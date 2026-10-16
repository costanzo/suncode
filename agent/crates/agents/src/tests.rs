use super::*;
use std::collections::BTreeSet;

#[test]
fn catalog_has_six_unique_specialists_and_one_primary() {
    let names = specialists()
        .iter()
        .map(|agent| agent.name)
        .collect::<BTreeSet<_>>();
    let ids = specialists()
        .iter()
        .map(|agent| agent.id)
        .chain([PRIMARY.id])
        .collect::<BTreeSet<_>>();
    assert_eq!(specialists().len(), 6);
    assert_eq!(names.len(), 6);
    assert_eq!(ids.len(), 7, "the primary id is distinct from specialists");
    assert!(!names.contains(PRIMARY.name));
}

#[test]
fn specialists_are_restricted_and_the_primary_is_not() {
    for agent in specialists() {
        assert!(!agent.is_primary());
        assert!(!agent.can_delegate && !agent.can_ask_user);
        assert!(!agent.can_use_external_tools && !agent.can_use_skills);
        assert_eq!(agent.model_policy, ModelPolicy::InheritParent);
        assert_eq!(agent.mcp_policy(), "Not allowed");
        let ToolScope::Only(tools) = agent.tools else {
            panic!("{} must have an explicit tool list", agent.name);
        };
        assert!(!tools.contains(&"question"));
        assert!(!tools.contains(&"delegate_agent"));
        assert!(!tools.iter().any(|tool| tool.starts_with("mcp_")));
    }
    assert!(PRIMARY.is_primary());
    assert_eq!(PRIMARY.tools, ToolScope::Catalog);
    const { assert!(PRIMARY.can_delegate && PRIMARY.can_ask_user) };
    const { assert!(PRIMARY.can_use_external_tools && PRIMARY.can_use_skills) };
    assert_eq!(PRIMARY.tool_call_limit, None);
}

#[test]
fn catalog_assigns_the_exact_role_tool_allowlists_and_limits() {
    for agent in specialists() {
        match agent.name {
            "architect-agent" | "ui-ux-agent" | "product-agent" => {
                assert_eq!(agent.tools, ToolScope::Only(READ_TOOLS));
                assert_eq!(agent.tool_call_limit, Some(32));
            }
            "swe-agent" | "qa-agent" | "sre-agent" => {
                assert_eq!(agent.tools, ToolScope::Only(IMPLEMENTATION_TOOLS));
                assert_eq!(agent.tool_call_limit, Some(64));
            }
            name => panic!("unexpected built-in agent {name}"),
        }
    }
}

#[test]
fn lookup_resolves_sessions_without_an_agent_to_the_primary() {
    assert_eq!(by_id(None), Some(&PRIMARY));
    assert_eq!(by_id(Some(PRIMARY.id)), Some(&PRIMARY));
    assert_eq!(by_id(Some("builtin.swe.v1")).unwrap().name, "swe-agent");
    assert_eq!(by_id(Some("builtin.missing.v1")), None);
    assert_eq!(specialist_by_name("qa-agent").unwrap().id, "builtin.qa.v1");
    assert_eq!(specialist_by_name("primary"), None);
    assert!(ToolScope::Catalog.allows("anything"));
    assert!(ToolScope::Only(READ_TOOLS).allows("grep"));
    assert!(!ToolScope::Only(READ_TOOLS).allows("bash"));
}
