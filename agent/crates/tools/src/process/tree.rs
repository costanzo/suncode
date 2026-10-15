//! Process-tree termination and post-exit group reaping.

#[cfg(target_os = "windows")]
use super::spawn::CREATE_NO_WINDOW;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::process::Child;
#[cfg(target_os = "windows")]
use std::process::Command;

pub(super) fn terminate_process_tree(child: &mut Child) {
    #[cfg(target_os = "windows")]
    {
        let pid = child.id();
        let _ = Command::new("taskkill")
            .creation_flags(CREATE_NO_WINDOW)
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .status();
    }
    #[cfg(unix)]
    {
        let pid = child.id() as i32;
        // The child is placed in its own process group by configure_command.
        unsafe {
            let _ = libc::kill(-pid, libc::SIGKILL);
        }
    }
    let _ = child.kill();
}

/// Kills processes left in the leader's process group after the leader exited,
/// such as `cmd &`, so they cannot keep the output pipes open.
pub(super) fn reap_process_group(child: &Child) {
    #[cfg(unix)]
    {
        // The group id stays reserved while any member is alive, so this cannot
        // reach an unrelated group.
        unsafe {
            let _ = libc::kill(-(child.id() as i32), libc::SIGKILL);
        }
    }
    #[cfg(not(unix))]
    let _ = child;
}
