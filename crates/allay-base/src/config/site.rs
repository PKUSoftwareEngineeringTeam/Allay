use crate::config::get_allay_config;
use crate::data::{AllayData, AllayObject};
use crate::file::{read_file_string, workspace};
use crate::log::NoPanicUnwrap;
use std::sync::Arc;
use std::{path::PathBuf, sync::OnceLock};

pub const SITE_CONFIG_FILE: &str = "allay.toml";

pub const DEFAULT_SITE_CONFIG: &str = r#"# Default Allay site configuration
baseUrl = "http://your-site.com/"
title = "Your Site Title"
theme = "your-theme-name"
description = "A brief description of your site."
author = "Your Name"
[params]"#;

pub fn get_site_config() -> Arc<AllayObject> {
    static INSTANCE: OnceLock<Arc<AllayObject>> = OnceLock::new();

    INSTANCE
        .get_or_init(|| {
            if let Ok(config) = read_file_string(workspace(SITE_CONFIG_FILE))
                && let Ok(config) = AllayData::from_toml(&config)
            {
                return Arc::new(config);
            }

            Arc::new(
                AllayData::from_toml(DEFAULT_SITE_CONFIG).expect("Failed to parse default config"),
            )
        })
        .clone()
}

pub fn get_theme_path() -> &'static PathBuf {
    static INSTANCE: OnceLock<PathBuf> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        const DEFAULT_THEME_NAME: &str = "Axolotl";

        let dir = &get_allay_config().theme_dir;
        let chosen = get_site_config()
            .get("theme")
            .map_or(DEFAULT_THEME_NAME, |data| {
                data.as_str().expect_("Theme name must be a string")
            })
            .to_string();
        PathBuf::from(dir).join(chosen)
    })
}
