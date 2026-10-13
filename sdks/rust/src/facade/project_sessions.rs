use super::*;

impl AsyncAgentSdk {
    pub fn list_sessions(&self, project_id: &str) -> SdkResult<SessionsResult> {
        if self
            .state
            .store
            .project_by_id_for_user(&self.state.user_id, project_id)?
            .is_none()
        {
            return Err(BusinessError::missing("project"));
        }
        let sessions = self.state.store.sessions_for_project(project_id, true)?;
        let session_states = sessions
            .iter()
            .map(|session| {
                self.state
                    .store
                    .session_ui_state(&session.session_id)
                    .map(|state| (session.session_id.clone(), state))
            })
            .collect::<Result<std::collections::HashMap<_, _>, _>>()?;
        Ok(SessionsResult {
            project_id: project_id.to_string(),
            sessions,
            session_states,
        })
    }
}
