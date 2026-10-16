//! SunCode agent definitions: who an agent is and what it may do.
//!
//! The core harness runs every agent the same way and reads only these
//! definitions to decide an agent's prompt, tool scope, and capabilities.
//! Definitions can only narrow what the core allows: policy, approval, audit,
//! and the authority rules in the system prompt stay in the core.

mod builtin;

pub use builtin::{IMPLEMENTATION_TOOLS, READ_TOOLS};

/// Tools an agent is offered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolScope {
    /// The full primary catalog: built-in, provider, and turn-level tools.
    Catalog,
    /// Only these tool names.
    Only(&'static [&'static str]),
}

impl ToolScope {
    /// Whether `name` is offered under this scope.
    pub fn allows(&self, name: &str) -> bool {
        match self {
            Self::Catalog => true,
            Self::Only(tools) => tools.contains(&name),
        }
    }
}

/// Which model an agent runs on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelPolicy {
    /// The session's selected model.
    SessionModel,
    /// The model of the session that delegated to it.
    InheritParent,
}

impl ModelPolicy {
    pub const fn description(self) -> &'static str {
        match self {
            Self::SessionModel => "Uses the session model",
            Self::InheritParent => "Inherits parent session model",
        }
    }
}

/// One agent: identity, instructions, and capabilities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentDefinition {
    /// Stable id persisted with sessions; changes only with `version`.
    pub id: &'static str,
    /// Name the primary agent uses when delegating.
    pub name: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub version: i64,
    /// Role instructions added after the core's authority and workflow rules.
    pub instructions: &'static str,
    pub tools: ToolScope,
    pub model_policy: ModelPolicy,
    /// May delegate tasks to other agents.
    pub can_delegate: bool,
    /// May pause the turn to ask the user a question.
    pub can_ask_user: bool,
    /// May use MCP, Browser Use, and Computer Use tools.
    pub can_use_external_tools: bool,
    /// May load project and user Skills.
    pub can_use_skills: bool,
    /// Tool-call ceiling for one turn; `None` uses the project limit alone.
    pub tool_call_limit: Option<u32>,
}

impl AgentDefinition {
    /// Whether this agent is the primary agent of a session.
    pub fn is_primary(&self) -> bool {
        self.id == PRIMARY.id
    }

    pub const fn mcp_policy(&self) -> &'static str {
        if self.can_use_external_tools {
            "Allowed"
        } else {
            "Not allowed"
        }
    }
}

/// The primary agent of every session.
pub const PRIMARY: AgentDefinition = AgentDefinition {
    id: "builtin.primary.v1",
    name: "primary",
    display_name: "SunCode",
    description: "The session's primary coding agent.",
    version: 1,
    instructions: "",
    tools: ToolScope::Catalog,
    model_policy: ModelPolicy::SessionModel,
    can_delegate: true,
    can_ask_user: true,
    can_use_external_tools: true,
    can_use_skills: true,
    tool_call_limit: None,
};

/// Specialist agents the primary agent may delegate to.
pub fn specialists() -> &'static [AgentDefinition] {
    builtin::SPECIALISTS
}

/// The agent with `id`; `None` resolves to [`PRIMARY`].
pub fn by_id(id: Option<&str>) -> Option<&'static AgentDefinition> {
    match id {
        None => Some(&PRIMARY),
        Some(id) if id == PRIMARY.id => Some(&PRIMARY),
        Some(id) => specialists().iter().find(|agent| agent.id == id),
    }
}

/// The specialist the primary agent delegates to by `name`.
pub fn specialist_by_name(name: &str) -> Option<&'static AgentDefinition> {
    specialists().iter().find(|agent| agent.name == name)
}

#[cfg(test)]
mod tests;
