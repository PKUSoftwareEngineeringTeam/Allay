use crate::{EventBus, config::get_plugin_config};
use allay_base::data::AllayObject;
use std::sync::Arc;

pub struct PluginContext {
    pub event_bus: Arc<EventBus>,
}

impl PluginContext {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self { event_bus }
    }
}

pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn initialize(&self, context: PluginContext) -> anyhow::Result<()>;
    fn config(&self) -> Arc<AllayObject> {
        get_plugin_config(self.name())
    }
}
