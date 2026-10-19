//! Per-project session lifecycle: verify, start, control, stop, shutdown.

use super::storage::prepare_managed_directory;
use super::{BrowserRuntime, BrowserRuntimeState, BrowserSlot};
use crate::cdp::CdpSession;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use suncode_common::{logging, BusinessError};

impl BrowserRuntime {
    pub async fn verify(&self) -> Result<(), BusinessError> {
        logging::debug(
            "browser.verify",
            format!(
                "begin host_available={} enabled={} cdp_port={}",
                self.inner.host_available,
                self.inner.enabled.load(Ordering::SeqCst),
                self.inner.cdp_port
            ),
        );
        self.require_enabled()?;
        let verification = async {
            let session = self.connect_for_verification().await?;
            {
                let mut current = self.inner.verification_session.lock().await;
                if self.inner.closed.load(Ordering::Acquire) {
                    drop(current);
                    session.close().await;
                    return Err(shutting_down());
                }
                *current = Some(session.clone());
            }
            session.close().await;
            self.inner.verification_session.lock().await.take();
            Ok::<(), BusinessError>(())
        }
        .await;
        if let Err(error) = verification {
            logging::write_business_error("browser.verify", "runtime", &error, "runtime_check");
            self.inner.verified.store(false, Ordering::SeqCst);
            if let Ok(mut current) = self.inner.verification_error.write() {
                *current = Some(error.message.clone());
            }
            return Err(error);
        }
        logging::info("browser.verify", "runtime_verified=true");
        self.inner.verified.store(true, Ordering::SeqCst);
        if let Ok(mut error) = self.inner.verification_error.write() {
            *error = None;
        }
        Ok(())
    }

    pub async fn start_project(&self, project_id: &str) -> Result<(), BusinessError> {
        let lock = self
            .inner
            .start_locks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entry(project_id.to_string())
            .or_default()
            .clone();
        let _guard = lock.lock().await;
        self.start_project_locked(project_id).await
    }

    pub(super) async fn start_project_locked(&self, project_id: &str) -> Result<(), BusinessError> {
        self.require_enabled()?;
        // CEF is host-owned; reaching this point is the integrity check.
        self.inner.verified.store(true, Ordering::SeqCst);
        {
            let mut projects = self.inner.projects.lock().await;
            if projects
                .get(project_id)
                .is_some_and(|slot| slot.session.is_some())
            {
                return Ok(());
            }
            projects.insert(project_id.into(), BrowserSlot::starting());
        }
        let profile_path = self.profile_path(project_id);
        if let Err(error) = prepare_managed_directory(
            &profile_path,
            &self.inner.data_dir.join("cef").join("browser").join("profiles"),
        ) {
            self.fail_slot(project_id, &error).await;
            return Err(error);
        }
        let session = match self.connect_project(project_id).await {
            Ok(session) => session,
            Err(error) => {
                self.fail_slot(project_id, &error).await;
                return Err(error);
            }
        };
        let installed = {
            let mut projects = self.inner.projects.lock().await;
            if self.inner.closed.load(Ordering::Acquire) {
                false
            } else if let Some(slot) = projects.get_mut(project_id) {
                slot.session = Some(session.clone());
                true
            } else {
                false
            }
        };
        if !installed {
            session.close().await;
            return Err(shutting_down());
        }
        // Reading the page proves the CDP connection is usable.
        if let Err(error) = session.state().await {
            session.close().await;
            self.fail_slot(project_id, &error).await;
            return Err(error);
        }
        if self.inner.closed.load(Ordering::Acquire) {
            session.close().await;
            return Err(shutting_down());
        }
        let mut projects = self.inner.projects.lock().await;
        if self.inner.closed.load(Ordering::Acquire) || !projects.contains_key(project_id) {
            drop(projects);
            session.close().await;
            return Err(shutting_down());
        }
        projects.insert(project_id.into(), BrowserSlot::running(session));
        Ok(())
    }

    /// Refreshes the project page state. The agent always holds control: there
    /// is no user lease yet, so taking and returning control are equivalent.
    pub async fn take_control(&self, project_id: &str) -> Result<(), BusinessError> {
        self.refresh(project_id).await
    }

    pub async fn return_control(&self, project_id: &str) -> Result<(), BusinessError> {
        self.refresh(project_id).await
    }

    pub async fn restart(&self, project_id: &str) -> Result<(), BusinessError> {
        self.stop(project_id).await?;
        self.start_project(project_id).await
    }

    pub async fn stop(&self, project_id: &str) -> Result<(), BusinessError> {
        let session = {
            let mut projects = self.inner.projects.lock().await;
            projects.remove(project_id).and_then(|slot| slot.session)
        };
        if let Some(session) = session {
            session.close().await;
        }
        Ok(())
    }

    pub async fn shutdown(&self) {
        if self.inner.closed.swap(true, Ordering::AcqRel) {
            return;
        }
        let sessions = {
            let mut projects = self.inner.projects.lock().await;
            projects
                .drain()
                .filter_map(|(_, slot)| slot.session)
                .collect::<Vec<_>>()
        };
        if let Some(session) = self.inner.verification_session.lock().await.take() {
            session.close().await;
        }
        for session in sessions {
            session.close().await;
        }
    }

    async fn refresh(&self, project_id: &str) -> Result<(), BusinessError> {
        let session = self.session(project_id).await?;
        session.state().await?;
        if let Some(slot) = self.inner.projects.lock().await.get_mut(project_id) {
            slot.state = BrowserRuntimeState::Background;
            slot.error = None;
        }
        Ok(())
    }

    pub(super) async fn session(&self, project_id: &str) -> Result<Arc<CdpSession>, BusinessError> {
        self.inner
            .projects
            .lock()
            .await
            .get(project_id)
            .and_then(|slot| slot.session.clone())
            .ok_or_else(|| {
                BusinessError::new(
                    "browser_runtime_not_started",
                    "Project browser is not running",
                )
            })
    }

    pub(super) async fn fail_slot(&self, project_id: &str, error: &BusinessError) {
        if self.inner.closed.load(Ordering::Acquire) {
            return;
        }
        let mut projects = self.inner.projects.lock().await;
        projects.insert(project_id.into(), BrowserSlot::failed(error));
    }
}

pub(super) fn shutting_down() -> BusinessError {
    BusinessError::new("agent_shutting_down", "agent shutdown is in progress")
}
