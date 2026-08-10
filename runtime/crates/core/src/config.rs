use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub data_dir: PathBuf,
    pub database_path: PathBuf,
    pub deepseek_endpoint: String,
    pub deepseek_model: String,
    pub zhipu_endpoint: String,
    pub zhipu_model: String,
    pub openai_endpoint: String,
    pub openai_model: String,
    pub non_interactive: bool,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let data_dir = std::env::var_os("SUNCODE_DATA_DIRECTORY")
            .map(PathBuf::from)
            .unwrap_or_else(default_data_dir);
        let database_path = std::env::var_os("SUNCODE_DATABASE_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|| data_dir.join("data/sqlite/runtime.sqlite3"));
        let non_interactive = env_bool("SUNCODE_NON_INTERACTIVE", false)?;
        if !non_interactive
            && [
                "DEEPSEEK_API_KEY",
                "ZHIPU_API_KEY",
                "ZAI_API_KEY",
                "OPENAI_API_KEY",
            ]
            .iter()
            .any(|name| std::env::var_os(name).is_some())
        {
            return Err(
                "provider API key environment overrides require SUNCODE_NON_INTERACTIVE=true"
                    .to_string(),
            );
        }
        Ok(Self {
            host: std::env::var("SUNCODE_RUNTIME_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("SUNCODE_RUNTIME_PORT")
                .ok()
                .map(|value| value.parse::<u16>())
                .transpose()
                .map_err(|_| "SUNCODE_RUNTIME_PORT must be an unsigned port".to_string())?
                .unwrap_or(0),
            data_dir,
            database_path,
            deepseek_endpoint: std::env::var("SUNCODE_DEEPSEEK_ENDPOINT")
                .unwrap_or_else(|_| "https://api.deepseek.com".to_string())
                .trim_end_matches('/')
                .to_string(),
            deepseek_model: std::env::var("SUNCODE_DEEPSEEK_MODEL")
                .unwrap_or_else(|_| "deepseek-v4-flash".to_string()),
            zhipu_endpoint: std::env::var("SUNCODE_ZHIPU_ENDPOINT")
                .unwrap_or_else(|_| "https://open.bigmodel.cn/api/paas/v4".to_string())
                .trim_end_matches('/')
                .to_string(),
            zhipu_model: std::env::var("SUNCODE_ZHIPU_MODEL")
                .unwrap_or_else(|_| "glm-5.2".to_string()),
            openai_endpoint: std::env::var("SUNCODE_OPENAI_ENDPOINT")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string())
                .trim_end_matches('/')
                .to_string(),
            openai_model: std::env::var("SUNCODE_OPENAI_MODEL")
                .unwrap_or_else(|_| "gpt-5.6-sol".to_string()),
            non_interactive,
        })
    }
}

fn env_bool(name: &str, fallback: bool) -> Result<bool, String> {
    let Ok(value) = std::env::var(name) else {
        return Ok(fallback);
    };
    match value.to_ascii_lowercase().as_str() {
        "1" | "true" => Ok(true),
        "0" | "false" => Ok(false),
        _ => Err(format!("{name} must be true or false")),
    }
}

fn default_data_dir() -> PathBuf {
    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".suncode")
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
}
