//! Continuous stdout/stderr draining with a bounded preview and spill file.

use super::{OUTPUT_SEQUENCE, PREVIEW_BYTES};
use crate::context::ProcessOutputCallback;
use std::collections::VecDeque;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::Ordering;

/// How long output readers may keep draining after the process has exited.
pub(super) const OUTPUT_DRAIN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);

/// Joins an output reader, giving up once `deadline` passes so a descendant
/// that still holds the pipe cannot block the tool call forever.
pub(super) fn join_output(
    reader: Option<std::thread::JoinHandle<CapturedOutput>>,
    deadline: std::time::Instant,
) -> Option<std::thread::Result<CapturedOutput>> {
    let reader = reader?;
    while !reader.is_finished() {
        if std::time::Instant::now() >= deadline {
            // The detached thread ends when the remaining writer closes the pipe.
            return None;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    Some(reader.join())
}

/// Spill file bytes kept per stream; output beyond this is counted but not stored.
const MAX_SPILL_BYTES: usize = 64 * 1024 * 1024;
const READ_CHUNK_BYTES: usize = 8192;

#[derive(Default)]
pub(super) struct CapturedOutput {
    pub(super) preview: Vec<u8>,
    pub(super) total_bytes: usize,
    pub(super) path: Option<PathBuf>,
    /// False when the spill file stopped short of `total_bytes`.
    pub(super) spill_complete: bool,
}

impl Drop for CapturedOutput {
    fn drop(&mut self) {
        if let Some(path) = self.path.take() {
            let _ = fs::remove_file(path);
        }
    }
}

pub(super) struct OutputCapture {
    preview: VecDeque<u8>,
    total_bytes: usize,
    path: Option<PathBuf>,
    file: Option<fs::File>,
    spilled_bytes: usize,
    /// Set once the spill file is capped or a write fails; the stream keeps
    /// draining so the child never blocks on a full pipe.
    spill_stopped: bool,
    spill_limit: usize,
    label: &'static str,
}

impl OutputCapture {
    fn new(label: &'static str) -> Self {
        Self {
            preview: VecDeque::with_capacity(PREVIEW_BYTES),
            total_bytes: 0,
            path: None,
            file: None,
            spilled_bytes: 0,
            spill_stopped: false,
            spill_limit: MAX_SPILL_BYTES,
            label,
        }
    }

    fn create_full_output(&mut self) -> std::io::Result<()> {
        let path = std::env::temp_dir().join(format!(
            "suncode-process-{}-{}-{}.tmp",
            std::process::id(),
            OUTPUT_SEQUENCE.fetch_add(1, Ordering::Relaxed),
            self.label
        ));
        // Record the path first so a failed write still removes the file.
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)?;
        self.path = Some(path);
        let preview = self.preview.make_contiguous();
        file.write_all(preview)?;
        self.spilled_bytes = preview.len();
        self.file = Some(file);
        Ok(())
    }

    fn spill(&mut self, bytes: &[u8]) {
        if self.spill_stopped {
            return;
        }
        if self.file.is_none()
            && self.total_bytes > PREVIEW_BYTES
            && self.create_full_output().is_err()
        {
            self.stop_spill();
            return;
        }
        let Some(file) = self.file.as_mut() else {
            return;
        };
        let room = self.spill_limit.saturating_sub(self.spilled_bytes);
        let chunk = &bytes[..bytes.len().min(room)];
        if file.write_all(chunk).is_err() {
            self.stop_spill();
            return;
        }
        self.spilled_bytes += chunk.len();
        if chunk.len() < bytes.len() {
            self.stop_spill();
        }
    }

    fn stop_spill(&mut self) {
        self.spill_stopped = true;
        self.file = None;
    }

    fn push(&mut self, bytes: &[u8]) {
        self.total_bytes = self.total_bytes.saturating_add(bytes.len());
        self.spill(bytes);
        self.preview.extend(bytes);
        while self.preview.len() > PREVIEW_BYTES {
            self.preview.pop_front();
        }
    }

    fn finish(mut self) -> CapturedOutput {
        if let Some(file) = self.file.as_mut() {
            if file.flush().is_err() {
                self.spill_stopped = true;
            }
        }
        CapturedOutput {
            preview: self.preview.drain(..).collect(),
            total_bytes: self.total_bytes,
            path: self.path.take(),
            spill_complete: !self.spill_stopped,
        }
    }
}

impl Drop for OutputCapture {
    fn drop(&mut self) {
        self.file = None;
        if let Some(path) = self.path.take() {
            let _ = fs::remove_file(path);
        }
    }
}

/// Drains `reader` to the end. Read errors end the stream early; the result
/// is still returned so the caller can report what was captured.
pub(super) fn collect_output(
    reader: impl Read,
    label: &'static str,
    callback: Option<ProcessOutputCallback>,
) -> CapturedOutput {
    drain(reader, OutputCapture::new(label), callback)
}

fn drain(
    mut reader: impl Read,
    mut output: OutputCapture,
    callback: Option<ProcessOutputCallback>,
) -> CapturedOutput {
    let label = output.label;
    let mut buffer = [0u8; READ_CHUNK_BYTES];
    loop {
        let bytes = match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => {
                output.spill_stopped = true;
                break;
            }
        };
        if let Some(callback) = callback.as_ref() {
            // Chunks are bounded by the fixed read buffer; callbacks are best effort and
            // must never interfere with process capture or completion.
            callback(label, &buffer[..bytes]);
        }
        output.push(&buffer[..bytes]);
    }
    output.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capture_with_limit(limit: usize) -> OutputCapture {
        let mut capture = OutputCapture::new("test");
        capture.spill_limit = limit;
        capture
    }

    #[test]
    fn spill_file_is_capped_but_all_bytes_are_counted() {
        let total = PREVIEW_BYTES + 4096;
        let limit = PREVIEW_BYTES + 1024;
        let output = drain(&vec![b'x'; total][..], capture_with_limit(limit), None);
        assert_eq!(output.total_bytes, total);
        assert!(!output.spill_complete);
        let path = output.path.as_ref().unwrap();
        assert_eq!(fs::metadata(path).unwrap().len() as usize, limit);
    }

    #[test]
    fn small_output_stays_in_memory_and_is_complete() {
        let output = collect_output(&b"hello"[..], "test", None);
        assert_eq!(output.preview, b"hello");
        assert!(output.path.is_none());
        assert!(output.spill_complete);
    }

    #[test]
    fn read_error_ends_capture_as_incomplete() {
        struct Failing(bool);
        impl Read for Failing {
            fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
                if std::mem::replace(&mut self.0, true) {
                    return Err(std::io::Error::other("broken pipe"));
                }
                buffer[..2].copy_from_slice(b"ok");
                Ok(2)
            }
        }
        let output = collect_output(Failing(false), "test", None);
        assert_eq!(output.preview, b"ok");
        assert!(!output.spill_complete);
    }
}
