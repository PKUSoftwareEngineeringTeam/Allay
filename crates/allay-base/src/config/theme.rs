use crate::config::get_theme_path;
use crate::file;
use crate::log::show_error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeMeta {
    pub name: String,
    pub version: String,
    pub author: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConfig {
    #[serde(rename = "custom", default = "FileConfig::default_custom_dir")]
    pub custom_dir: String,
    #[serde(rename = "static", default = "FileConfig::default_static_dir")]
    pub static_dir: String,
    #[serde(default)]
    pub templates: TemplateConfig,
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            custom_dir: "content".to_string(),
            static_dir: "static".to_string(),
            templates: TemplateConfig::default(),
        }
    }
}

impl FileConfig {
    fn default_custom_dir() -> String {
        "content".to_string()
    }

    fn default_static_dir() -> String {
        "static".to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Dependencies {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub plugins: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    #[serde(default = "FileConfig::default_dir")]
    pub dir: String,
    #[serde(default = "FileConfig::default_index")]
    pub index: String,
    #[serde(default = "FileConfig::default_content")]
    pub content: String,
    #[serde(default = "FileConfig::default_not_found")]
    pub not_found: String,
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            dir: "templates".to_string(),
            index: "index.html".to_string(),
            content: "page.html".to_string(),
            not_found: "404.html".to_string(),
        }
    }
}

impl FileConfig {
    fn default_dir() -> String {
        "templates".to_string()
    }

    fn default_index() -> String {
        "index.html".to_string()
    }

    fn default_content() -> String {
        "page.html".to_string()
    }

    fn default_not_found() -> String {
        "404.html".to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    #[serde(rename = "theme")]
    pub meta: ThemeMeta,
    #[serde(rename = "config", default)]
    pub config: FileConfig,
    #[serde(default)]
    pub dependencies: Dependencies,
}

pub fn get_theme_config() -> &'static ThemeConfig {
    static THEME_CONFIG: OnceLock<ThemeConfig> = OnceLock::new();

    THEME_CONFIG.get_or_init(|| {
        let theme_path = file::workspace(get_theme_path());
        let config_file = theme_path.join("theme.toml");
        let config_str = file::read_file_string(&config_file)
            .unwrap_or_else(|e| show_error(&format!("Failed to read theme config: {e}")));
        toml::from_str(&config_str).unwrap_or_else(|_| show_error("Failed to parse theme config"))
    })
}
