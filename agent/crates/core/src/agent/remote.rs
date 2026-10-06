impl Agent {
    pub fn remote_snapshot(&self) -> Result<serde_json::Value, BusinessError> {
        let projects = self.store.projects_for_user(&self.user_id, false)?;
        let mut sessions = Vec::new();
        let mut session_states = serde_json::Map::new();
        for project in &projects {
            for session in self.store.sessions_for_project(&project.project_id, true)? {
                session_states.insert(
                    session.session_id.clone(),
                    serde_json::Value::String(self.store.session_ui_state(&session.session_id)?),
                );
                sessions.push(session);
            }
        }
        Ok(serde_json::json!({
            "projects": projects,
            "sessions": sessions,
            "sessionStates": session_states,
        }))
    }

    pub async fn remote_dispatch(
        &self,
        request_id: &str,
        operation: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, BusinessError> {
        let string = |snake: &str, camel: &str| {
            arguments
                .get(snake)
                .or_else(|| arguments.get(camel))
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| BusinessError::invalid(format!("{camel} is required")))
        };
        match operation {
            "projects.list" => {
                let snapshot = self.remote_snapshot()?;
                Ok(serde_json::json!({"projects": snapshot["projects"]}))
            }
            "sessions.list" => {
                let project_id = string("project_id", "projectId")?;
                if project_id.is_empty() {
                    let snapshot = self.remote_snapshot()?;
                    Ok(serde_json::json!({
                        "project_id": "",
                        "sessions": snapshot["sessions"],
                        "sessionStates": snapshot["sessionStates"],
                    }))
                } else {
                    let project = self
                        .store
                        .project_by_id_for_user(&self.user_id, project_id)?
                        .ok_or_else(|| BusinessError::missing("project"))?;
                    let sessions = self.store.sessions_for_project(&project.project_id, true)?;
                    let mut states = serde_json::Map::new();
                    for session in &sessions {
                        states.insert(
                            session.session_id.clone(),
                            serde_json::Value::String(self.store.session_ui_state(&session.session_id)?),
                        );
                    }
                    Ok(serde_json::json!({"project_id": project_id, "sessions": sessions, "sessionStates": states}))
                }
            }
            "session.create" => {
                let project_id = string("project_id", "projectId")?;
                self.store
                    .project_by_id_for_user(&self.user_id, project_id)?
                    .ok_or_else(|| BusinessError::missing("project"))?;
                let title = arguments.get("title").and_then(serde_json::Value::as_str);
                Ok(serde_json::to_value(self.store.create_session(project_id, title, None)?)?)
            }
            "session.get" => {
                let session_id = string("session_id", "sessionId")?;
                let session = self.remote_session(session_id)?;
                Ok(serde_json::json!({
                    "session": session,
                    "messages": self.store.messages(session_id)?,
                    "conversationTurns": self.store.session_conversation_turns(session_id)?,
                    "images": self.store.session_images(session_id)?,
                    "pendingQuestion": self.store.pending_question(session_id)?,
                    "pendingApproval": self.store.pending_approval(session_id)?.map(|value| serde_json::to_value(value)).transpose()?,
                }))
            }
            "session.send_message" | "session.message" => {
                let session_id = string("session_id", "sessionId")?;
                let text = string("text", "text")?;
                let response = self.submit_with_attachments(
                    session_id,
                    request_id,
                    text,
                    arguments.get("model").and_then(serde_json::Value::as_str),
                    arguments.get("reasoning_effort").and_then(serde_json::Value::as_str),
                    &[],
                ).await?;
                Ok(serde_json::to_value(response)?)
            }
            "session.cancel" | "turn.cancel" => {
                let session_id = string("session_id", "sessionId")?;
                let turn_id = arguments.get("turn_id").or_else(|| arguments.get("turnId")).and_then(serde_json::Value::as_str).map(str::to_owned).or_else(|| self.store.session_conversation_turns(session_id).ok().and_then(|turns| turns.last().map(|turn| turn.turn_id.clone()))).ok_or_else(|| BusinessError::invalid("turnId is required"))?;
                Ok(serde_json::json!({"turnId": turn_id, "status": if self.cancel(&turn_id) { "cancellation_requested" } else { "not_running" }}))
            }
            "session.retry" | "turn.retry" => Ok(serde_json::to_value(self.retry_last_turn(string("session_id", "sessionId")?).await?)?),
            "approval.resolve" => {
                let approval_id = string("approval_id", "approvalId")?;
                let decision = string("decision", "action")?;
                if !["deny", "allow_once", "allow_session"].contains(&decision) { return Err(BusinessError::invalid("invalid approval decision")); }
                Ok(serde_json::json!({"approvalId": approval_id, "decision": decision, "resolved": self.resolve_approval(approval_id, decision).await?}))
            }
            "question.reply" => {
                let request_id = string("question_id", "questionId")?;
                let answers = arguments.get("answers").and_then(serde_json::Value::as_array).ok_or_else(|| BusinessError::invalid("answers must be an array"))?;
                let answers = answers.iter().map(|answer| answer.as_array().map(|values| values.iter().filter_map(serde_json::Value::as_str).map(str::to_owned).collect()).ok_or_else(|| BusinessError::invalid("each answer must be an array"))).collect::<Result<Vec<Vec<String>>, _>>()?;
                Ok(serde_json::json!({"requestId": request_id, "status": if self.resolve_question(request_id, answers, false).await? { "replied" } else { "conflict" }}))
            }
            _ => Err(BusinessError::invalid("Remote operation is not supported")),
        }
    }

    fn remote_session(&self, session_id: &str) -> Result<crate::domain::SessionRecord, BusinessError> {
        let session = self.store.session_by_id(session_id)?.ok_or_else(|| BusinessError::missing("session"))?;
        let project_id = session.project_id.as_deref().ok_or_else(|| BusinessError::new("scope_denied", "session is not bound to a project"))?;
        self.store.project_by_id_for_user(&self.user_id, project_id)?.ok_or_else(|| BusinessError::missing("session"))?;
        Ok(session)
    }
}
