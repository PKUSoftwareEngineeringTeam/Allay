use serde::{Deserialize, Serialize};

const fn default_port() -> u16 {
    3000
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_static_files_path() -> String {
    "./static".to_string()
}

/// Configuration for the backend server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    /// Server port, default is 3000
    #[serde(default = "default_port")]
    pub port: u16,
    /// Server host, default is `127.0.0.1`
    #[serde(default = "default_host")]
    pub host: String,
    /// Path to static files, default is `./static`
    #[serde(default = "default_static_files_path")]
    pub static_files_path: String,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            host: default_host(),
            static_files_path: default_static_files_path(),
        }
    }
}

impl BackendConfig {
    /// Load configuration from a TOML file.
    pub fn from_config_file(path: &str) -> anyhow::Result<Self> {
        let config_str = std::fs::read_to_string(path)?;
        let config: BackendConfig = toml::from_str(&config_str)?;
        Ok(config)
    }
}
