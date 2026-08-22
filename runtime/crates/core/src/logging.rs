use chrono::{Local, SecondsFormat};
use std::{
    env,
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

struct Logger {
    minimum_level: Level,
    file: Mutex<Option<File>>,
}

static LOGGER: OnceLock<Arc<Logger>> = OnceLock::new();

pub(crate) fn initialize(data_dir: &Path) {
    let _ = LOGGER.get_or_init(|| {
        let minimum_level = Level::parse(env::var("SUNCODE_LOG_LEVEL").ok().as_deref());
        let directory = env::var_os("SUNCODE_LOG_DIRECTORY")
            .map(PathBuf::from)
            .unwrap_or_else(|| data_dir.join("logs"));
        let file = open_log_file(&directory.join("runtime.log"));
        Arc::new(Logger {
            minimum_level,
            file: Mutex::new(file),
        })
    });
}

pub(crate) fn write(level: Level, component: &str, message: impl Display) {
    let Some(logger) = LOGGER.get() else {
        eprintln!("[suncode][{}][{}] {}", level.name(), component, message);
        return;
    };
    if level < logger.minimum_level || logger.minimum_level == Level::Off {
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
    if let Ok(mut file) = logger.file.lock() {
        if let Some(file) = file.as_mut() {
            if writeln!(file, "{line}").and_then(|_| file.flush()).is_err() {
                eprintln!("[suncode][ERROR][logger] file_write_failed");
            }
        }
    }
    eprintln!("{line}");
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
