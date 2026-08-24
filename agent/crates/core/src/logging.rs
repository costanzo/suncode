use chrono::{Local, SecondsFormat};
use std::{
    fmt::Display,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
    thread,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum Level {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Off = 5,
}

impl Level {
    fn parse(value: Option<&str>) -> Self {
        match value.unwrap_or("INFO").trim().to_ascii_uppercase().as_str() {
            "TRACE" => Self::Trace,
            "DEBUG" => Self::Debug,
            "WARN" | "WARNING" => Self::Warn,
            "ERROR" => Self::Error,
            "OFF" | "NONE" => Self::Off,
            _ => Self::Info,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
            Self::Off => "OFF",
        }
    }
}

struct LoggerState {
    minimum_level: Level,
    file_path: PathBuf,
    max_bytes: u64,
    retention: usize,
    file: Option<File>,
}

struct Logger {
    state: Mutex<LoggerState>,
}

static LOGGER: OnceLock<Arc<Logger>> = OnceLock::new();

pub(crate) struct Config<'a> {
    pub level: &'a str,
    pub directory: Option<&'a str>,
    pub max_bytes: u64,
    pub retention: usize,
}

pub(crate) fn configure(data_dir: &Path, config: Config<'_>) {
    let directory = config
        .directory
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| data_dir.join("logs"));
    let file_path = directory.join("agent.log");
    let state = LoggerState {
        minimum_level: Level::parse(Some(config.level)),
        file: open_log_file(&file_path),
        file_path,
        max_bytes: config.max_bytes.max(1024),
        retention: config.retention.min(100),
    };
    if let Some(logger) = LOGGER.get() {
        if let Ok(mut current) = logger.state.lock() {
            *current = state;
        }
    } else {
        let _ = LOGGER.set(Arc::new(Logger {
            state: Mutex::new(state),
        }));
    }
}

pub(crate) fn write(level: Level, component: &str, message: impl Display) {
    let Some(logger) = LOGGER.get() else {
        eprintln!("[suncode][{}][{}] {}", level.name(), component, message);
        return;
    };
    let Ok(mut state) = logger.state.lock() else {
        return;
    };
    if level < state.minimum_level || state.minimum_level == Level::Off {
        return;
    }

    let line = format!(
        "[suncode][{}][{}][pid={}][tid={:?}][{}] {}",
        Local::now().to_rfc3339_opts(SecondsFormat::Millis, false),
        level.name(),
        std::process::id(),
        thread::current().id(),
        component,
        message
    );
    let should_rotate = state
        .file
        .as_ref()
        .and_then(|value| value.metadata().ok())
        .is_some_and(|metadata| metadata.len() + line.len() as u64 + 1 > state.max_bytes);
    if should_rotate {
        let file_path = state.file_path.clone();
        let retention = state.retention;
        if let Err(error) = rotate_log(&file_path, &mut state.file, retention) {
            eprintln!("[suncode][ERROR][logger] file_rotate_failed error={error}");
        }
    }
    if let Some(file) = state.file.as_mut() {
        if writeln!(file, "{line}").and_then(|_| file.flush()).is_err() {
            eprintln!("[suncode][ERROR][logger] file_write_failed");
        }
    }
    eprintln!("{line}");
}

fn rotate_log(path: &Path, file: &mut Option<File>, retention: usize) -> io::Result<()> {
    let _ = file.take();
    if retention == 0 {
        if path.exists() {
            fs::remove_file(path)?;
        }
    } else {
        let oldest = path.with_extension(format!("log.{retention}"));
        if oldest.exists() {
            fs::remove_file(oldest)?;
        }
        for index in (1..retention).rev() {
            let source = path.with_extension(format!("log.{index}"));
            if source.exists() {
                fs::rename(source, path.with_extension(format!("log.{}", index + 1)))?;
            }
        }
        if path.exists() {
            fs::rename(path, path.with_extension("log.1"))?;
        }
    }
    *file = open_log_file(path);
    if file.is_none() {
        return Err(io::Error::other("log file could not be reopened"));
    }
    Ok(())
}

fn open_log_file(path: &Path) -> Option<File> {
    if let Some(parent) = path.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            eprintln!(
                "[suncode][ERROR][logger] directory_create_failed path={parent:?} error={error}"
            );
            return None;
        }
    }
    let result = OpenOptions::new()
        .create(true)
        .append(true)
        .read(true)
        .open(path);
    if let Err(error) = &result {
        eprintln!("[suncode][ERROR][logger] file_open_failed path={path:?} error={error}");
        return None;
    }
    let file = result.ok()?;
    restrict_permissions(path).ok();
    Some(file)
}

#[cfg(unix)]
fn restrict_permissions(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotates_active_file_and_keeps_bounded_backups() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("agent.log");
        fs::write(&path, "active").unwrap();
        fs::write(path.with_extension("log.1"), "older").unwrap();
        let mut file = open_log_file(&path);

        rotate_log(&path, &mut file, 2).unwrap();

        assert_eq!(
            fs::read_to_string(path.with_extension("log.1")).unwrap(),
            "active"
        );
        assert_eq!(
            fs::read_to_string(path.with_extension("log.2")).unwrap(),
            "older"
        );
        assert!(path.exists());
    }
}
