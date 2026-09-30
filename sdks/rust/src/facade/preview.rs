use super::*;

impl AsyncAgentSdk {
    pub fn preview_state(&self, project_id: &str) -> SdkResult<suncode_agent::PreviewState> {
        self.project_for_user(project_id)?;
        Ok(self.state.agent.preview_state(project_id))
    }

    pub fn start_preview(&self, project_id: &str, program: &str, args: Vec<String>, cwd: Option<String>, url: &str) -> SdkResult<suncode_agent::PreviewState> {
        let project = self.project_for_user(project_id)?;
        self.state.agent.start_preview(project_id, &project.canonical_root, program, args, cwd, url)
    }

    pub fn stop_preview(&self, project_id: &str) -> SdkResult<suncode_agent::PreviewState> {
        self.project_for_user(project_id)?;
        Ok(self.state.agent.stop_preview(project_id))
    }
}
