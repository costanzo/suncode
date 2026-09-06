use super::*;

impl AgentSdk {
    pub fn list_sessions(&self, project_id: &str) -> SdkResult<SessionsResult> {
        if self.state.store.project_by_id(project_id)?.is_none() {
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

    pub fn git_status(&self, project_id: &str) -> SdkResult<GitStatusResult> {
        let project = self
            .state
            .store
            .project_by_id(project_id)?
            .ok_or_else(|| BusinessError::missing("project"))?;
        let value = self
            .state
            .operations
            .execute_in_project(
                std::path::Path::new(&project.canonical_root),
                "git/status",
                json!({}),
            )
            .map_err(operation_error)?;
        decode_operation(value, "git/status")
    }

    pub fn git_diff_file(
        &self,
        project_id: &str,
        scope: &str,
        path: &str,
    ) -> SdkResult<GitDiffFileResult> {
        if !matches!(scope, "all" | "staged" | "unstaged") {
            return Err(BusinessError::invalid(
                "scope must be all, staged, or unstaged",
            ));
        }
        if path.trim().is_empty() {
            return Err(BusinessError::invalid("path is required"));
        }
        let project = self
            .state
            .store
            .project_by_id(project_id)?
            .ok_or_else(|| BusinessError::missing("project"))?;
        let value = self
            .state
            .operations
            .execute_in_project(
                std::path::Path::new(&project.canonical_root),
                "git/diff-file",
                json!({"scope": scope, "path": path}),
            )
            .map_err(operation_error)?;
        decode_operation(value, "git/diff-file")
    }
}
