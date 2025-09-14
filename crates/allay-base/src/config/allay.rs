use config::Config;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllayConfig {
    pub name: String,
    pub description: String,
    pub version: String,
    pub repository: String,
    pub content: ContentConfig,
    pub publish: PublishConfig,
    pub statics: StaticConfig,
    pub template: TemplateConfig,
    pub theme: ThemeConfig,
    pub log: LogConfig,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentConfig {
    pub dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishConfig {
    pub dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticConfig {
    pub dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub dir: String,
    pub default: DefaultThemeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultThemeConfig {
    pub name: String,
    pub repository: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub dir: String,
}

fn get_environment() -> Environment {
    let env = std::env::var("ALLAY_ENV")
        .unwrap_or_else(|_| "production".into())
        .to_lowercase();
    match env.as_str() {
        "dev" | "development" => Environment::Development,
        _ => Environment::Production,
    }
}

fn load_allay_config() -> AllayConfig {
    let config = Config::builder()
        .add_source(config::File::with_name("config/allay-config.toml"))
        .build()
        .unwrap();

    config.try_deserialize().unwrap()
}

// Configs exposed
pub static ENVRIONMENT: LazyLock<Environment> = LazyLock::new(get_environment);
pub static ALLAY_CONFIG: LazyLock<AllayConfig> = LazyLock::new(load_allay_config);
