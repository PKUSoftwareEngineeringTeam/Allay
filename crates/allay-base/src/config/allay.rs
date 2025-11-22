use crate::file;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllayConfig {
    #[serde(default = "AllayConfig::default_content_dir")]
    pub content_dir: String,
    #[serde(default = "AllayConfig::default_publish_dir")]
    pub publish_dir: String,
    #[serde(default = "AllayConfig::default_statics_dir")]
    pub statics_dir: String,
    #[serde(default = "AllayConfig::default_plugin_dir")]
    pub plugin_dir: String,
    #[serde(default = "AllayConfig::default_shortcode_dir")]
    pub shortcode_dir: String,
    #[serde(default = "AllayConfig::default_theme_dir")]
    pub theme_dir: String,
    #[serde(default = "AllayConfig::default_log_dir")]
    pub log_dir: String,
}

impl Default for AllayConfig {
    fn default() -> Self {
        Self {
            content_dir: Self::default_content_dir(),
            publish_dir: Self::default_publish_dir(),
            statics_dir: Self::default_statics_dir(),
            plugin_dir: Self::default_plugin_dir(),
            shortcode_dir: Self::default_shortcode_dir(),
            theme_dir: Self::default_theme_dir(),
            log_dir: Self::default_log_dir(),
        }
    }
}

impl AllayConfig {
    fn default_content_dir() -> String {
        "contents".into()
    }

    fn default_publish_dir() -> String {
        "public".into()
    }

    fn default_statics_dir() -> String {
        "static".into()
    }

    fn default_plugin_dir() -> String {
        "plugins".into()
    }

    fn default_shortcode_dir() -> String {
        "shortcodes".into()
    }

    fn default_theme_dir() -> String {
        "themes".into()
    }

    fn default_log_dir() -> String {
        "logs".into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Production,
}

impl Environment {
    /// Check if the environment is development
    pub fn is_dev(&self) -> bool {
        matches!(self, Environment::Development)
    }

    /// Check if the environment is production
    pub fn is_prod(&self) -> bool {
        matches!(self, Environment::Production)
    }
}

pub fn get_env() -> &'static Environment {
    static INSTANCE: OnceLock<Environment> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        let env = std::env::var("ALLAY_ENV").unwrap_or("production".into()).to_lowercase();
        match env.as_str() {
            "dev" | "development" => Environment::Development,
            _ => Environment::Production,
        }
    })
}

pub fn get_allay_config() -> &'static AllayConfig {
    static INSTANCE: OnceLock<AllayConfig> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        let config_file = std::env::home_dir().map(|p| p.join(".config/allay/config.toml"));
        if let Some(config_file) = config_file
            && let Ok(config) = file::read_file_string(config_file)
        {
            toml::from_str(&config).unwrap_or_default()
        } else {
            AllayConfig::default()
        }
    })
}
