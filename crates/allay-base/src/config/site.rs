use std::sync::OnceLock;

use crate::{
    data::{AllayData, AllayObject},
    file::{read_file_string, workspace},
};

pub const SITE_CONFIG_FILE: &str = "allay.toml";

pub const DEFAULT_SITE_CONFIG: &str = r#"# Default Allay site configuration
baseUrl = "http://your-site.com/"
title = "Your Site Title"
description = "A brief description of your site."
author = "Your Name"
[params]"#;

pub fn get_site_config() -> &'static AllayObject {
    static INSTANCE: OnceLock<AllayObject> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        if let Ok(config) = read_file_string(workspace(SITE_CONFIG_FILE))
            && let Ok(config) = AllayData::from_toml(&config)
        {
            return config;
        }

        AllayData::from_toml(DEFAULT_SITE_CONFIG).expect("Failed to parse default config")
    })
}
