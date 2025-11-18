pub mod config;
pub mod manager;
pub mod types;

use allay_base::{config::get_allay_config, file};
pub use manager::PluginManager;
use std::path::Path;
use tracing::{info, warn};

pub fn load_plugins() {
    let dir = &get_allay_config().plugin.dir;

    let manager = PluginManager::instance();

    // find all .wasm files in the plugin directory and register them
    match file::read_files(file::workspace(dir)) {
        Err(e) => {
            warn!("Failed to read plugin directory {}: {}", dir, e);
        }

        Ok(paths) => {
            for path in paths {
                if let Some(ext) = path.extension()
                    && ext == "wasm"
                {
                    match manager.register_plugin(&path, Path::new(dir)) {
                        Ok(()) => info!("Registered plugin from {}", path.to_string_lossy()),
                        Err(e) => warn!(
                            "Failed to register plugin from {}: {}",
                            path.to_string_lossy(),
                            e
                        ),
                    }
                }
            }
        }
    }
}
