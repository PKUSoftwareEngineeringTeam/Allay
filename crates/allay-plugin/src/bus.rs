use anymap::{Map, any::Any};
use std::sync::{Arc, RwLock};
use tracing::warn;

pub trait Event {}

pub trait EventHandler<E: Event>: Send + Sync {
    fn handle_event(&self, event: &mut E) -> anyhow::Result<()>;
}

struct GenericEventBus<E: Event> {
    handlers: Vec<Arc<dyn EventHandler<E>>>,
}

impl<E: Event> GenericEventBus<E> {
    fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }
}

/// An `AnyMap` to store the bus with generic parameters
pub struct EventBus(RwLock<Map<dyn Any + Send + Sync>>);

impl Default for EventBus {
    fn default() -> Self {
        Self(RwLock::new(Map::new()))
    }
}

impl EventBus {
    pub fn register_handler<E: Event + 'static>(&self, handler: Arc<dyn EventHandler<E>>) {
        let mut buses = self.0.write().unwrap();
        buses.entry().or_insert_with(GenericEventBus::new).handlers.push(handler);
    }

    pub fn publish<E: Event + 'static>(&self, event: &mut E) {
        let buses = self.0.read().unwrap();
        if let Some(bus) = buses.get::<GenericEventBus<E>>() {
            for handler in bus.handlers.iter() {
                if let Err(e) = handler.handle_event(event) {
                    warn!("Error handling event: {}", e);
                }
            }
        }
    }
}
