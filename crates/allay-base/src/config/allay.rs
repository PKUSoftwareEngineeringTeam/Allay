use config::Config;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllayConfig {
    pub name: String,
    pub description: String,
    pub version: String,
    pub repository: String,
    pub env: Environment,
    content: ContentConfig,
    publish: PublishConfig,
    statics: StaticConfig,
    template: TemplateConfig,
    theme: ThemeConfig,
    log: LogConfig,
}

impl AllayConfig {
    pub fn is_dev(&self) -> bool {
        matches!(self.env, Environment::Development)
    }

    pub fn is_prod(&self) -> bool {
        matches!(self.env, Environment::Production)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Production,
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

static ALLAY_CONFIG: LazyLock<AllayConfig> = LazyLock::new(load_allay_config);

fn load_allay_config() -> AllayConfig {
    let env = std::env::var("ALLAY_ENV").unwrap_or("prod".into());
    let config = match env.as_str() {
        "dev" | "development" => "config/dev.toml",
        "prod" | "production" => "config/prod.toml",
        _ => "config/prod.toml",
    };

    let config = Config::builder()
        .add_source(config::File::with_name("config/base.toml"))
        .add_source(config::File::with_name(config).required(false))
        .build()
        .unwrap();

    config.try_deserialize().unwrap()
}

// Configs exposed
pub static GLOBAL_CONFIG: LazyLock<&AllayConfig> = LazyLock::new(|| &ALLAY_CONFIG);
pub static CONTENT_CONFIG: LazyLock<&ContentConfig> = LazyLock::new(|| &ALLAY_CONFIG.content);
pub static PUBLISH_CONFIG: LazyLock<&PublishConfig> = LazyLock::new(|| &ALLAY_CONFIG.publish);
pub static STATIC_CONFIG: LazyLock<&StaticConfig> = LazyLock::new(|| &ALLAY_CONFIG.statics);
pub static TEMPLATE_CONFIG: LazyLock<&TemplateConfig> = LazyLock::new(|| &ALLAY_CONFIG.template);
pub static THEME_CONFIG: LazyLock<&ThemeConfig> = LazyLock::new(|| &ALLAY_CONFIG.theme);
pub static LOG_CONFIG: LazyLock<&LogConfig> = LazyLock::new(|| &ALLAY_CONFIG.log);
