use std::sync::LazyLock;

use crate::{
    data::{AllayData, AllayObject},
    file::{read_file, workspace},
};

pub const SITE_CONFIG_FILE: &str = "allay.toml";

pub const DEFAULT_SITE_CONFIG: &str = r#"# Default Allay site configuration
baseUrl = "http://your-site.com/"
title = "Your Site Title"
description = "A brief description of your site."
author = "Your Name"
[params]"#;

pub static SITE_CONFIG: LazyLock<AllayObject> = LazyLock::new(load_site_config);

fn load_site_config() -> AllayObject {
    if let Ok(config) = read_file(workspace(SITE_CONFIG_FILE))
        && let Ok(config) = AllayData::from_toml(&config.content)
    {
        return config;
    }

    AllayData::from_toml(DEFAULT_SITE_CONFIG).expect("Failed to parse default config")
}
