use crate::EventBus;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    async fn initialize(&self, context: PluginContext) -> anyhow::Result<()>;
}

pub struct PluginContext {
    pub event_bus: Arc<EventBus>,
}

impl PluginContext {
    pub fn new(event_bus: Arc<EventBus>) -> Self {
        Self { event_bus }
    }
}
