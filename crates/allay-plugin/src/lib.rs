pub mod config;
pub mod manager;
pub mod types;

use allay_base::{config::get_allay_config, file};
pub use manager::{Plugin, PluginManager};
use tracing::{info, warn};

pub fn load_plugins() {
    let dir = &get_allay_config().plugin_dir;
    let dir = file::workspace(dir);

    let manager = PluginManager::instance();

    // find all .wasm files in the plugin directory and register them
    if let Ok(paths) = file::read_files(&dir) {
        for path in paths {
            if let Some(ext) = path.extension()
                && ext == "wasm"
            {
                if let Err(e) = manager.register_plugin(&path, &file::absolute_root()) {
                    eprintln!("Failed to register plugin from {:?}: {}", path, e)
                } else {
                    info!("Registered plugin from {:?}", path);
                }
            }
        }
    } else {
        warn!("Failed to read plugin directory {:?}", dir);
    }
}
