use crate::EventBus;
use std::sync::Arc;

pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn initialize(&self, context: PluginContext) -> anyhow::Result<()>;
}

pub struct PluginContext {
    pub event_bus: Arc<EventBus>,
}

impl PluginContext {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self { event_bus }
    }
}
