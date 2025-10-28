use allay_base::data::{AllayData, AllayObject};
use allay_base::file;
use std::sync::{Arc, OnceLock};
use tracing::warn;

const PLUGIN_CONFIG_FILE: &str = "plugin.toml";

pub fn get_plugin_config(name: &str) -> Arc<AllayObject> {
    static PLUGINS_CONFIG: OnceLock<Arc<AllayObject>> = OnceLock::new();
    let default = Arc::new(AllayObject::default());
    let configs = PLUGINS_CONFIG
        .get_or_init(|| match file::read_file_string(PLUGIN_CONFIG_FILE) {
            Ok(content) => AllayData::from_toml(&content).map(Arc::new).unwrap_or_else(|e| {
                warn!("Failed to parse config for plugin '{}': {}", name, e);
                default.clone()
            }),
            Err(e) => {
                warn!("Failed to read config file for plugin '{}': {}", name, e);
                default.clone()
            }
        })
        .clone();

    match configs.get(name) {
        Some(data) => data.as_obj().unwrap_or_else(|_| {
            warn!("Config for plugin '{}' is not an object", name);
            default
        }),
        None => {
            warn!("No config found for plugin '{}'", name);
            default
        }
    }
}
