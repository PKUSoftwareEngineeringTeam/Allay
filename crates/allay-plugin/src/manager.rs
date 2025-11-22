use allay_base::log::NoPanicUnwrap;
use allay_plugin_host::PluginHost;
use semver::{Version, VersionReq};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock, RwLock};

pub type Plugin = Arc<Mutex<PluginHost>>;

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

    pub fn register_plugin(&self, wasm_path: &Path, working_dir: &Path) -> anyhow::Result<()> {
        let mut host = PluginHost::new(wasm_path, working_dir)?;
        let name = host.plugin_name()?;
        let mut plugins = self.plugins.write().expect_("Failed to acquire write lock on plugins");

        plugins.insert(name, Arc::new(Mutex::new(host)));
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

    pub fn plugin_names(&self) -> Vec<String> {
        let plugins = self.plugins.read().unwrap();
        plugins.keys().cloned().collect()
    }

    pub fn version_match(&self, name: &str, req_version: &str) -> anyhow::Result<bool> {
        let req = VersionReq::parse(req_version)?;
        let plugins = self.plugins.read().expect_("failed to acquire read lock on plugins");
        if let Some(plugin) = plugins.get(name) {
            let mut plugin = plugin.lock().expect_("failed to acquire lock on plugin");
            let version = plugin.plugin_version()?;
            let version = Version::parse(&version)?;
            Ok(req.matches(&version))
        } else {
            Ok(false)
        }
    }
}
