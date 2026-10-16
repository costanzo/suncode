//! Process-tree ownership, termination, and post-exit reaping.
//!
//! On Unix the child leads its own process group (see `configure_command`).
//! On Windows it is placed in a kill-on-close Job Object, with `taskkill /T`
//! as a fallback for descendants created before assignment.

#[cfg(target_os = "windows")]
use super::job::ProcessJob;
#[cfg(target_os = "windows")]
use super::spawn::CREATE_NO_WINDOW;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::process::Child;
#[cfg(target_os = "windows")]
use std::process::Command;

/// A spawned child together with whatever owns its descendants.
pub(super) struct ProcessTree {
    pub(super) child: Child,
    #[cfg(target_os = "windows")]
    job: Option<ProcessJob>,
}

impl ProcessTree {
    pub(super) fn new(child: Child) -> Self {
        Self {
            #[cfg(target_os = "windows")]
            job: ProcessJob::assign(&child),
            child,
        }
    }

    /// Kills the child and every descendant that can still be reached.
    pub(super) fn terminate(&mut self) {
        #[cfg(target_os = "windows")]
        {
            if let Some(job) = self.job.as_ref() {
                job.terminate();
            }
            let pid = self.child.id();
            let _ = Command::new("taskkill")
                .creation_flags(CREATE_NO_WINDOW)
                .args(["/F", "/T", "/PID", &pid.to_string()])
                .status();
        }
        #[cfg(unix)]
        {
            let pid = self.child.id() as i32;
            // SAFETY: signalling a process group has no memory-safety
            // requirements; the child leads its own group.
            unsafe {
                let _ = libc::kill(-pid, libc::SIGKILL);
            }
        }
        let _ = self.child.kill();
    }

    /// Kills descendants left behind after the child exited, such as
    /// `cmd &`, so they cannot keep the output pipes open.
    pub(super) fn reap(&self) {
        #[cfg(unix)]
        {
            // The group id stays reserved while any member is alive, so this
            // cannot reach an unrelated group.
            // SAFETY: as in `terminate`.
            unsafe {
                let _ = libc::kill(-(self.child.id() as i32), libc::SIGKILL);
            }
        }
        #[cfg(target_os = "windows")]
        if let Some(job) = self.job.as_ref() {
            job.terminate();
        }
    }
}
