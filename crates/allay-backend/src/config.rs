use serde::{Deserialize, Serialize};

const fn default_port() -> u16 {
    3000
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_static_files_path() -> String {
    "./static".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_static_files_path")]
    pub static_files_path: String,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            host: "0.0.0.0".to_string(),
            static_files_path: "./static".to_string(),
        }
    }
}

impl BackendConfig {
    pub fn from_config_file(path: &str) -> anyhow::Result<Self> {
        let config_str = std::fs::read_to_string(path)?;
        let config: BackendConfig = toml::from_str(&config_str)?;
        Ok(config)
    }
}
