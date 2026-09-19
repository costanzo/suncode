use super::*;

impl AsyncAgentSdk {
    pub fn list_agents(&self) -> SdkResult<AgentsResult> {
        Ok(AgentsResult {
            agents: suncode_agent::agent::builtin_agents::all()
                .iter()
                .map(|agent| BuiltinAgentDto {
                    id: agent.id.into(),
                    name: agent.name.into(),
                    display_name: agent.display_name.into(),
                    description: agent.description.into(),
                    version: agent.version,
                    allowed_tools: agent
                        .allowed_tools
                        .iter()
                        .map(|tool| (*tool).into())
                        .collect(),
                    model_policy: agent.model_policy.into(),
                    mcp_policy: agent.mcp_policy.into(),
                    can_delegate: agent.can_delegate,
                    tool_call_limit: agent.tool_call_limit,
                })
                .collect(),
        })
    }

    pub fn list_child_sessions(&self, parent_session_id: &str) -> SdkResult<ChildSessionsResult> {
        let parent = self.session_for_user(parent_session_id)?;
        if parent.kind != "primary" {
            return Err(BusinessError::invalid("parent session must be primary"));
        }
        let sessions = self
            .state
            .store
            .child_sessions_for_parent(parent_session_id)?;
        let session_states = sessions
            .iter()
            .map(|session| {
                self.state
                    .store
                    .session_ui_state(&session.session_id)
                    .map(|state| (session.session_id.clone(), state))
            })
            .collect::<Result<std::collections::HashMap<_, _>, _>>()?;
        Ok(ChildSessionsResult {
            parent_session_id: parent_session_id.into(),
            sessions,
            session_states,
            invocations: self
                .state
                .store
                .subagent_invocations_for_parent(parent_session_id)?,
        })
    }
}
