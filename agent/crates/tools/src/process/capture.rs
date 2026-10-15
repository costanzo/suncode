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
    reader: Option<std::thread::JoinHandle<std::io::Result<CapturedOutput>>>,
    deadline: std::time::Instant,
) -> Option<std::thread::Result<std::io::Result<CapturedOutput>>> {
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

pub(super) struct CapturedOutput {
    pub(super) preview: Vec<u8>,
    pub(super) total_bytes: usize,
    pub(super) path: Option<PathBuf>,
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
    label: &'static str,
}

impl OutputCapture {
    fn new(label: &'static str) -> Self {
        Self {
            preview: VecDeque::with_capacity(PREVIEW_BYTES),
            total_bytes: 0,
            path: None,
            file: None,
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
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)?;
        file.write_all(self.preview.make_contiguous())?;
        self.preview.clear();
        self.path = Some(path);
        self.file = Some(file);
        Ok(())
    }

    fn push(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        self.total_bytes = self.total_bytes.saturating_add(bytes.len());
        if self.file.is_none() && self.total_bytes > PREVIEW_BYTES {
            self.create_full_output()?;
        }
        if let Some(file) = self.file.as_mut() {
            file.write_all(bytes)?;
        }
        self.preview.extend(bytes);
        while self.preview.len() > PREVIEW_BYTES {
            self.preview.pop_front();
        }
        Ok(())
    }

    fn finish(mut self) -> std::io::Result<CapturedOutput> {
        if let Some(file) = self.file.as_mut() {
            file.flush()?;
        }
        Ok(CapturedOutput {
            preview: self.preview.into_iter().collect(),
            total_bytes: self.total_bytes,
            path: self.path.take(),
        })
    }
}

pub(super) fn collect_output(
    mut reader: impl Read,
    label: &'static str,
    callback: Option<ProcessOutputCallback>,
) -> std::io::Result<CapturedOutput> {
    let mut output = OutputCapture::new(label);
    let mut buffer = [0u8; 8192];
    loop {
        let bytes = reader.read(&mut buffer)?;
        if bytes == 0 {
            break;
        }
        if let Some(callback) = callback.as_ref() {
            // Chunks are bounded by the fixed read buffer; callbacks are best effort and
            // must never interfere with process capture or completion.
            callback(label, &buffer[..bytes]);
        }
        output.push(&buffer[..bytes])?;
    }
    output.finish()
}
