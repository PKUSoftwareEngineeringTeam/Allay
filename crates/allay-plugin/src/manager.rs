use allay_plugin_host::PluginHost;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, OnceLock, RwLock};

pub type Plugin = Arc<PluginHost>;

/// Manager for plugins.
/// Handles registration and retrieval of plugins
#[derive(Default)]
pub struct PluginManager {
    plugins: RwLock<HashMap<String, Plugin>>,
}

impl PluginManager {
    pub fn instance() -> &'static Self {
        static INSTANCE: OnceLock<PluginManager> = OnceLock::new();
        INSTANCE.get_or_init(PluginManager::default)
    }

    pub fn register_plugin(&self, wasm_path: impl AsRef<Path>) -> anyhow::Result<()> {
        let host = PluginHost::new(wasm_path)?;
        let name = host.plugin_name()?;
        let mut plugins = self.plugins.write().expect("Failed to acquire write lock on plugins");

        plugins.insert(name, Arc::new(host));
        Ok(())
    }

    pub fn get_plugin(&self, name: &str) -> Option<Plugin> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(name).cloned()
    }

    pub fn plugins(&self) -> Vec<Plugin> {
        let plugins = self.plugins.read().unwrap();
        plugins.values().cloned().collect()
    }
}
