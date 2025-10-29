use crate::{EventBus, Plugin, PluginContext};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

/// Manager for plugins.
/// Handles registration and retrieval of plugins
#[derive(Default)]
pub struct PluginManager {
    plugins: RwLock<HashMap<String, Arc<dyn Plugin>>>,
    event_bus: Arc<EventBus>,
}

impl PluginManager {
    pub fn instance() -> Arc<Self> {
        static INSTANCE: OnceLock<Arc<PluginManager>> = OnceLock::new();
        INSTANCE.get_or_init(|| Arc::new(PluginManager::default())).clone()
    }

    pub fn event_bus(&self) -> Arc<EventBus> {
        self.event_bus.clone()
    }

    pub fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> anyhow::Result<()> {
        let name = plugin.name().to_string();
        let mut plugins = self.plugins.write().unwrap();

        if plugins.contains_key(&name) {
            return Err(anyhow::anyhow!("Plugin '{}' is already registered", name));
        }

        let context = PluginContext::new(self.event_bus.clone());
        plugin.initialize(context)?;

        plugins.insert(name, plugin);
        Ok(())
    }

    pub fn get_plugin(&self, name: &str) -> Option<Arc<dyn Plugin>> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(name).cloned()
    }

    pub fn list_plugins(&self) -> Vec<String> {
        let plugins = self.plugins.read().unwrap();
        plugins.keys().cloned().collect()
    }
}
