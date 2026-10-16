//! Long-lived project processes owned by the host.

use super::spawn::{command_arguments, configure_command, process_cwd, process_start_failure};
use super::tree::ProcessTree;
use crate::arguments::ProcessArguments;
use crate::scope::require_project;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use suncode_common::BusinessError;

/// A project-scoped process that remains alive until the host explicitly stops it.
pub struct BackgroundProcess {
    tree: Mutex<ProcessTree>,
}

impl BackgroundProcess {
    pub fn stop(&self) {
        if let Ok(mut tree) = self.tree.lock() {
            tree.terminate();
            let _ = tree.child.wait();
        }
    }

    pub fn is_running(&self) -> bool {
        self.tree
            .lock()
            .map(|mut tree| tree.child.try_wait().ok().flatten().is_none())
            .unwrap_or(false)
    }
}

impl Drop for BackgroundProcess {
    fn drop(&mut self) {
        self.stop();
    }
}

pub fn start_background(
    project_root: &Path,
    args: &ProcessArguments,
) -> Result<BackgroundProcess, BusinessError> {
    let root = require_project(Some(project_root))?;
    let (program, command_args) = command_arguments(args)?;
    let cwd = process_cwd(root, args)?;
    let child = configure_command(Command::new(&program), args)
        .args(command_args)
        .current_dir(cwd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(process_start_failure)?;
    Ok(BackgroundProcess {
        tree: Mutex::new(ProcessTree::new(child)),
    })
}
