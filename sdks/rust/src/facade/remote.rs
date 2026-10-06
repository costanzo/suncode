use super::*;
use futures_util::StreamExt;
use std::{future::Future, pin::Pin};
use suncode_remote::{RemoteHost, RemoteNetworkConfiguration, RemoteResult, RemoteSessionEvent};
use tokio::sync::mpsc;

pub(crate) struct SdkRemoteHost {
    pub(crate) state: AgentState,
}

impl RemoteHost for SdkRemoteHost {
    fn network_configuration(&self) -> RemoteNetworkConfiguration {
        RemoteNetworkConfiguration {
            verify_https_certificates: self
                .state
                .verify_https_certificates
                .load(std::sync::atomic::Ordering::SeqCst),
            use_system_certificates: self
                .state
                .use_system_certificates
                .load(std::sync::atomic::Ordering::SeqCst),
            certificate_path: self
                .state
                .certificate_path
                .read()
                .ok()
                .and_then(|p| p.clone()),
            proxy: self
                .state
                .proxy_configuration
                .read()
                .map(|p| p.clone())
                .unwrap_or_default(),
        }
    }

    fn snapshot(&self) -> RemoteResult<Value> {
        self.state.agent.remote_snapshot()
    }

    fn subscribe_session_events(
        &self,
        session_id: &str,
    ) -> RemoteResult<mpsc::Receiver<RemoteResult<RemoteSessionEvent>>> {
        let sdk = AsyncAgentSdk::from_state(self.state.clone());
        let mut stream = sdk.subscribe_session_events(session_id)?;
        let (sender, receiver) = mpsc::channel(64);
        let session_id = session_id.to_owned();
        tokio::spawn(async move {
            while let Some(event) = stream.next().await {
                let item = event
                    .map_err(|error| BusinessError::unavailable(error.to_string()))
                    .map(|event| RemoteSessionEvent {
                        session_id: session_id.clone(),
                        occurred_at: event.occurred_at.clone(),
                        event_type: event.event_type().as_str().to_owned(),
                        payload: event.payload.clone().into_value(),
                    });
                if sender.send(item).await.is_err() {
                    break;
                }
            }
        });
        Ok(receiver)
    }

    fn dispatch<'a>(
        &'a self,
        request_id: &'a str,
        operation: &'a str,
        arguments: Value,
    ) -> Pin<Box<dyn Future<Output = RemoteResult<Value>> + Send + 'a>> {
        Box::pin(async move {
            self.state
                .agent
                .remote_dispatch(request_id, operation, arguments)
                .await
        })
    }
}

impl AsyncAgentSdk {
    pub fn remote_server_configuration(&self) -> RemoteServerConfiguration {
        self.remote
            .as_ref()
            .map(|remote| remote.configuration())
            .unwrap_or(RemoteServerConfiguration {
                server_url: String::new(),
                pairing_code: String::new(),
                e2e_enabled: true,
            })
    }

    pub fn remote_server_status(&self) -> RemoteServerStatus {
        self.remote
            .as_ref()
            .map(|remote| remote.status())
            .unwrap_or(RemoteServerStatus {
                configured: false,
                connected: false,
                connecting: false,
                host_id: None,
                mobile_pairing_payload: None,
                access_token_expires_at: None,
                mobile_pairing_code: None,
                mobile_pairing_url: None,
                error: None,
            })
    }

    pub fn save_remote_server_configuration(
        &self,
        configuration: RemoteServerConfiguration,
    ) -> SdkResult<RemoteServerStatus> {
        let remote = self.remote.as_ref().ok_or_else(|| {
            BusinessError::unavailable("Remote Server is unavailable in this SDK view")
        })?;
        remote.save_configuration(configuration)?;
        Ok(remote.status())
    }

    pub async fn connect_remote_server(&self) -> SdkResult<RemoteServerStatus> {
        let remote = self.remote.as_ref().ok_or_else(|| {
            BusinessError::unavailable("Remote Server is unavailable in this SDK view")
        })?;
        match remote.connect().await {
            Ok(status) => Ok(status),
            Err(error) => {
                remote.set_status_for_error(error.message.clone());
                Err(error)
            }
        }
    }

    pub fn disconnect_remote_server(&self) -> SdkResult<RemoteServerStatus> {
        self.remote
            .as_ref()
            .ok_or_else(|| {
                BusinessError::unavailable("Remote Server is unavailable in this SDK view")
            })?
            .disconnect()
    }

    pub fn clear_remote_server_configuration(&self) -> SdkResult<RemoteServerStatus> {
        self.remote
            .as_ref()
            .ok_or_else(|| {
                BusinessError::unavailable("Remote Server is unavailable in this SDK view")
            })?
            .clear()
    }
}

#[allow(dead_code)]
async fn dispatch_remote_request(
    sdk: &AsyncAgentSdk,
    request_id: &str,
    operation: &str,
    arguments: Value,
) -> SdkResult<Value> {
    let string = |snake: &str, camel: &str| {
        arguments
            .get(snake)
            .or_else(|| arguments.get(camel))
            .and_then(Value::as_str)
            .ok_or_else(|| BusinessError::invalid(format!("{camel} is required")))
    };
    match operation {
        "projects.list" => Ok(serde_json::to_value(sdk.list_projects()?)?),
        "sessions.list" => {
            let project_id = string("project_id", "projectId")?;
            if project_id.is_empty() {
                let mut sessions = Vec::new();
                let mut states = std::collections::HashMap::new();
                for project in sdk.list_projects()?.projects {
                    let result = sdk.list_sessions(&project.project_id)?;
                    sessions.extend(result.sessions);
                    states.extend(result.session_states);
                }
                Ok(json!({"project_id":"","sessions":sessions,"sessionStates":states}))
            } else {
                Ok(serde_json::to_value(sdk.list_sessions(project_id)?)?)
            }
        }
        "session.create" => Ok(serde_json::to_value(sdk.create_session(
            string("project_id", "projectId")?,
            arguments.get("title").and_then(Value::as_str),
            None,
        )?)?),
        "session.get" => Ok(serde_json::to_value(
            sdk.session_snapshot(string("session_id", "sessionId")?, 0)?,
        )?),
        "session.send_message" | "session.message" => Ok(serde_json::to_value(
            sdk.submit_turn(
                string("session_id", "sessionId")?,
                string("text", "text")?,
                request_id,
                arguments.get("model").and_then(Value::as_str),
                arguments.get("reasoning_effort").and_then(Value::as_str),
            )
            .await?,
        )?),
        "session.cancel" | "turn.cancel" => {
            let session_id = string("session_id", "sessionId")?;
            let snapshot = sdk.session_snapshot(session_id, 0)?;
            let turn_id = arguments
                .get("turn_id")
                .or_else(|| arguments.get("turnId"))
                .and_then(Value::as_str)
                .or_else(|| {
                    snapshot
                        .conversation_turns
                        .last()
                        .map(|turn| turn.turn_id.as_str())
                })
                .ok_or_else(|| BusinessError::invalid("turnId is required"))?;
            Ok(serde_json::to_value(sdk.cancel_turn(session_id, turn_id)?)?)
        }
        "session.retry" | "turn.retry" => Ok(serde_json::to_value(
            sdk.retry_last_turn(string("session_id", "sessionId")?)
                .await?,
        )?),
        "approval.resolve" => Ok(serde_json::to_value(
            sdk.resolve_approval(
                string("approval_id", "approvalId")?,
                string("decision", "action")?,
            )
            .await?,
        )?),
        "question.reply" => {
            let answers = arguments
                .get("answers")
                .cloned()
                .ok_or_else(|| BusinessError::invalid("answers are required"))?;
            let answers = if answers
                .as_array()
                .is_some_and(|v| v.iter().all(Value::is_string))
            {
                Value::Array(
                    answers
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|value| Value::Array(vec![value.clone()]))
                        .collect(),
                )
            } else {
                answers
            };
            Ok(serde_json::to_value(
                sdk.reply_question(string("question_id", "questionId")?, &answers)
                    .await?,
            )?)
        }
        _ => Err(BusinessError::invalid("Remote operation is not supported")),
    }
}
