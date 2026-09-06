use super::*;
use std::{ffi::CString, os::raw::c_void, thread::JoinHandle};
pub struct AgentSubscription {
    pub(super) session_id: String,
    pub(super) cancellation: CancellationToken,
    pub(super) join: Mutex<Option<JoinHandle<()>>>,
}

impl AgentSubscription {
    pub(crate) fn close(&self) {
        logging::write(
            Level::Debug,
            "subscription_close",
            format!("cancel_begin session={}", self.session_id),
        );
        self.cancellation.cancel();
        if let Ok(mut join) = self.join.lock() {
            if let Some(join) = join.take() {
                logging::write(
                    Level::Debug,
                    "subscription_close",
                    format!("join_begin session={}", self.session_id),
                );
                let _ = join.join();
                logging::write(
                    Level::Debug,
                    "subscription_close",
                    format!("join_end session={}", self.session_id),
                );
            }
        }
        logging::write(
            Level::Debug,
            "subscription_close",
            format!("end session={}", self.session_id),
        );
    }
}

impl Drop for AgentSubscription {
    fn drop(&mut self) {
        self.close();
    }
}
pub(super) fn emit_sdk_event(
    callback: SunCodeEventCallback,
    user_data: usize,
    event: &SessionEvent,
) {
    let Ok(value) = serde_json::to_string(event) else {
        logging::write(
            Level::Error,
            "sdk.event",
            "operation=serialize_event failed=true",
        );
        return;
    };
    let Ok(value) = CString::new(value) else {
        logging::write(
            Level::Error,
            "sdk.event",
            "operation=marshal_event failed=true",
        );
        return;
    };
    unsafe { callback(value.as_ptr(), user_data as *mut c_void) };
}
