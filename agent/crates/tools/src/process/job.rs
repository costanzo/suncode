//! Windows Job Objects that own a spawned process tree.
//!
//! `taskkill /T` follows parent-process links, so it misses descendants whose
//! parent already exited. Every process created inside a job stays in it, so
//! terminating the job reaches the whole tree, and closing the last handle
//! kills it too (`KILL_ON_JOB_CLOSE`), so a dropped tree cannot leak.
//!
//! The child is assigned right after spawn. A descendant created in that
//! window is not in the job; the `taskkill /T` fallback still covers it while
//! its parent is alive.

use std::os::windows::io::AsRawHandle;
use std::process::Child;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
    SetInformationJobObject, TerminateJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};

/// Exit code reported by processes ended through the job.
const TERMINATED_EXIT_CODE: u32 = 1;

pub(super) struct ProcessJob(HANDLE);

// SAFETY: a job handle is a kernel object reference usable from any thread;
// it is only closed in `Drop`.
unsafe impl Send for ProcessJob {}
unsafe impl Sync for ProcessJob {}

impl ProcessJob {
    /// Creates a kill-on-close job and assigns `child` to it. Returns `None`
    /// when either step fails, leaving `taskkill` as the only mechanism.
    pub(super) fn assign(child: &Child) -> Option<Self> {
        // SAFETY: null attributes and name create an unnamed job with default
        // security; the returned handle is owned by `ProcessJob`.
        let handle = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if handle.is_null() {
            return None;
        }
        let job = Self(handle);
        // SAFETY: zero is a valid bit pattern for this plain C struct.
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: `limits` is a valid JOBOBJECT_EXTENDED_LIMIT_INFORMATION of
        // the size passed, and `job.0` is a live job handle.
        let configured = unsafe {
            SetInformationJobObject(
                job.0,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        if configured == 0 {
            return None;
        }
        // SAFETY: both handles are live; `child` keeps its process handle open.
        let assigned = unsafe { AssignProcessToJobObject(job.0, child.as_raw_handle() as HANDLE) };
        (assigned != 0).then_some(job)
    }

    /// Terminates every process in the job.
    pub(super) fn terminate(&self) {
        // SAFETY: `self.0` is a live job handle.
        unsafe {
            TerminateJobObject(self.0, TERMINATED_EXIT_CODE);
        }
    }
}

impl Drop for ProcessJob {
    fn drop(&mut self) {
        // SAFETY: the handle is owned and closed exactly once.
        unsafe {
            CloseHandle(self.0);
        }
    }
}
