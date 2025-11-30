use crate::generator::FileListener;
use allay_base::{config::get_allay_config, file::FileResult};
use allay_plugin::PluginManager;
use std::path::PathBuf;

pub struct PluginListener;

impl FileListener for PluginListener {
    fn root(&self) -> PathBuf {
        get_allay_config().content_dir.clone().into()
    }

    fn on_create(&self, path: PathBuf) -> FileResult<()> {
        let plugin_manager = PluginManager::instance();
        for plugin in plugin_manager.plugins() {
            if let Ok(mut plugin) = plugin.lock() {
                plugin.on_create(path.to_string_lossy().into());
            }
        }
        Ok(())
    }

    fn on_remove(&self, path: PathBuf) -> FileResult<()> {
        let plugin_manager = PluginManager::instance();
        for plugin in plugin_manager.plugins() {
            if let Ok(mut plugin) = plugin.lock() {
                plugin.on_remove(path.to_string_lossy().into());
            }
        }
        Ok(())
    }

    fn on_modify(&self, path: PathBuf) -> FileResult<()> {
        let plugin_manager = PluginManager::instance();
        for plugin in plugin_manager.plugins() {
            if let Ok(mut plugin) = plugin.lock() {
                plugin.on_modify(path.to_string_lossy().into());
            }
        }
        Ok(())
    }
}
