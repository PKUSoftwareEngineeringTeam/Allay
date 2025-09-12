use std::{path::Path, sync::OnceLock};

use crate::{
    costants::CONFIG_FILE,
    data::{AllayData, AllayObject},
    file::{read_file, workspace},
};

pub const DEFAULT_SITE_CONFIG: &str = r#"# Default Allay site configuration
baseUrl = "http://your-site.com/"
title = "Your Site Title"
description = "A brief description of your site."
author = "Your Name"
[params]"#;

pub static SITE_CONFIG: OnceLock<AllayObject> = OnceLock::new();

pub fn config_exists() -> bool {
    Path::new(CONFIG_FILE).exists()
}

pub fn load_site_config() {
    if let Ok(config) = read_file(workspace(CONFIG_FILE)) {
        if let Ok(config) = AllayData::from_toml(&config.content) {
            SITE_CONFIG.set(config).ok();
            return;
        }
    }

    tracing::warn!(
        "Failed to load config from {}. Using default config.",
        CONFIG_FILE
    );
    SITE_CONFIG
        .set(AllayData::from_toml(DEFAULT_SITE_CONFIG).expect("Failed to parse default config"))
        .ok();
}
