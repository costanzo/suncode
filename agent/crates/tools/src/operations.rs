//! Public operations API consumed by the agent core and SDK.

use crate::arguments::ProcessArguments;
use crate::context::{ExecutionContext, NetworkSettings, ProcessOutputCallback};
use crate::error::fatal;
use crate::process::{self, BackgroundProcess};
use crate::{dispatch, project_view, scope};
use serde_json::Value;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use suncode_common::{BusinessError, HttpProxyConfiguration};
use suncode_documents::Documents;

pub type CertificatePath = Arc<RwLock<Option<PathBuf>>>;
pub type ProxyConfiguration = Arc<RwLock<HttpProxyConfiguration>>;

pub struct Operations {
    checkpoint_root: PathBuf,
    verify_https_certificates: Arc<AtomicBool>,
    use_system_certificates: Arc<AtomicBool>,
    certificate_path: CertificatePath,
    proxy_configuration: ProxyConfiguration,
    /// Parsed documents shared by the document tools.
    documents: Documents,
}

impl Operations {
    pub fn new(checkpoint_root: PathBuf) -> io::Result<Self> {
        Self::new_with_https_certificate_verification(
            checkpoint_root,
            Arc::new(AtomicBool::new(true)),
        )
    }

    pub fn new_with_https_certificate_verification(
        checkpoint_root: PathBuf,
        verify_https_certificates: Arc<AtomicBool>,
    ) -> io::Result<Self> {
        std::fs::create_dir_all(&checkpoint_root)?;
        Ok(Self {
            checkpoint_root: checkpoint_root.canonicalize()?,
            verify_https_certificates,
            use_system_certificates: Arc::new(AtomicBool::new(true)),
            certificate_path: Arc::new(RwLock::new(None)),
            proxy_configuration: Arc::new(RwLock::new(HttpProxyConfiguration::default())),
            documents: Documents::new(),
        })
    }

    pub fn set_certificate_configuration(&self, use_system: bool, path: Option<PathBuf>) {
        self.use_system_certificates
            .store(use_system, Ordering::SeqCst);
        if let Ok(mut current) = self.certificate_path.write() {
            *current = path;
        }
    }

    pub fn set_proxy_configuration(&self, configuration: HttpProxyConfiguration) {
        if let Ok(mut current) = self.proxy_configuration.write() {
            *current = configuration;
        }
    }

    pub fn start_background_process(
        &self,
        project_path: &Path,
        params: Value,
    ) -> Result<BackgroundProcess, BusinessError> {
        let args: ProcessArguments = serde_json::from_value(params)
            .map_err(|_| BusinessError::invalid("background process arguments are invalid"))?;
        process::start_background(project_path, &args)
    }

    pub fn open_project(&self, project_path: &Path) -> Result<Value, BusinessError> {
        scope::open_project(project_path).map(|(_, value)| value)
    }

    pub fn list_directory(
        &self,
        root_path: &Path,
        relative_path: &str,
        max_entries: usize,
    ) -> Result<Value, BusinessError> {
        project_view::list_directory(root_path, relative_path, max_entries)
    }

    pub fn read_text_file(
        &self,
        root_path: &Path,
        relative_path: &str,
        max_bytes: usize,
    ) -> Result<Value, BusinessError> {
        project_view::read_text_file(root_path, relative_path, max_bytes)
    }

    pub fn execute_in_project(
        &self,
        project_path: &Path,
        method: &str,
        params: Value,
    ) -> Result<Value, BusinessError> {
        self.execute_in_project_with_cancellation_and_output(
            project_path,
            method,
            params,
            None,
            None,
        )
    }

    pub fn execute_in_project_with_cancellation_and_output(
        &self,
        project_path: &Path,
        method: &str,
        params: Value,
        cancellation: Option<&AtomicBool>,
        output_callback: Option<ProcessOutputCallback>,
    ) -> Result<Value, BusinessError> {
        let project_root = canonical_project(project_path)?;
        let context = ExecutionContext {
            project_root: Some(&project_root),
            checkpoint_root: Some(&self.checkpoint_root),
            cancellation,
            output_callback,
            network: self.network_settings(),
            documents: &self.documents,
        };
        dispatch::dispatch(method, &params, context)
    }

    fn network_settings(&self) -> NetworkSettings {
        NetworkSettings {
            verify_https_certificates: self.verify_https_certificates.load(Ordering::SeqCst),
            use_system_certificates: self.use_system_certificates.load(Ordering::SeqCst),
            certificate_path: self
                .certificate_path
                .read()
                .ok()
                .and_then(|path| path.clone()),
            proxy: self
                .proxy_configuration
                .read()
                .map(|configuration| configuration.clone())
                .unwrap_or_default(),
        }
    }
}

fn canonical_project(project_path: &Path) -> Result<PathBuf, BusinessError> {
    let canonical = project_path
        .canonicalize()
        .map_err(|_| fatal("project_unavailable", "project root is unavailable"))?;
    if !canonical.is_dir() {
        return Err(fatal(
            "project_unavailable",
            "project root is not a directory",
        ));
    }
    Ok(canonical)
}
