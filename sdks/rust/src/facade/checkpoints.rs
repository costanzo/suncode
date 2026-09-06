use super::*;

impl AgentSdk {
    pub fn list_checkpoints(&self, session_id: &str) -> SdkResult<CheckpointsResult> {
        Ok(CheckpointsResult {
            session_id: session_id.to_string(),
            checkpoints: self.state.store.manifests(session_id)?,
        })
    }

    pub fn checkpoint_manifest(&self, manifest_id: &str) -> SdkResult<CheckpointDetails> {
        let manifest = self
            .state
            .store
            .manifest(manifest_id)?
            .ok_or_else(|| BusinessError::missing("checkpoint"))?;
        Ok(CheckpointDetails {
            manifest,
            items: self.state.store.checkpoint_items(manifest_id)?,
        })
    }

    pub fn restore_checkpoint(
        &self,
        manifest_id: &str,
        session_id: &str,
    ) -> SdkResult<RestoreOutcome> {
        let manifest = self
            .state
            .store
            .manifest(manifest_id)?
            .ok_or_else(|| BusinessError::missing("checkpoint"))?;
        if manifest.session_id != session_id {
            return Err(BusinessError::new(
                "scope_denied",
                "checkpoint does not belong to session",
            ));
        }
        if manifest.status != "available" {
            return Err(BusinessError::new(
                "checkpoint_unavailable",
                "checkpoint is not available",
            ));
        }
        let session = self
            .state
            .store
            .session_by_id(session_id)?
            .ok_or_else(|| BusinessError::missing("session"))?;
        let project = self
            .state
            .store
            .project_by_id(session.project_id.as_deref().unwrap_or(""))?
            .ok_or_else(|| BusinessError::missing("project"))?;
        self.state
            .store
            .set_manifest_status(manifest_id, "restoring")?;
        let items = self.state.store.checkpoint_items(manifest_id)?;
        let mut restored = 0;
        for item in &items {
            if item.status != "available" {
                continue;
            }
            match self.state.operations.execute_in_project(
                std::path::Path::new(&project.canonical_root),
                "checkpoint/restore",
                json!({"checkpoint_id": item.checkpoint_id}),
            ) {
                Ok(result) => {
                    restored += 1;
                    emit_event(
                        &self.state,
                        session_id,
                        "checkpoint.item_restored",
                        json!({
                            "manifest_id": manifest_id,
                            "checkpoint_id": item.checkpoint_id,
                            "path": result.get("path")
                        }),
                    )?;
                }
                Err(error) => {
                    let status = if restored > 0 { "partial" } else { "conflict" };
                    self.state.store.set_manifest_status(manifest_id, status)?;
                    emit_event(
                        &self.state,
                        session_id,
                        "checkpoint.restore_failed",
                        json!({"manifest_id": manifest_id, "status": status, "code": error.get("code")}),
                    )?;
                    return Err(BusinessError::new(
                        error
                            .get("code")
                            .and_then(Value::as_str)
                            .unwrap_or("restore_conflict"),
                        error
                            .get("message")
                            .and_then(Value::as_str)
                            .unwrap_or("checkpoint restore failed"),
                    ));
                }
            }
        }
        self.state
            .store
            .set_manifest_status(manifest_id, "restored")?;
        emit_event(
            &self.state,
            session_id,
            "checkpoint.restored",
            json!({"manifest_id": manifest_id, "restored_items": restored}),
        )?;
        Ok(RestoreOutcome {
            manifest_id: manifest_id.to_string(),
            status: "restored",
            restored_items: restored,
        })
    }
}
