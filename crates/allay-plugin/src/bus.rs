use crate::Event;
use anymap::{Map, any::Any};
use async_trait::async_trait;
use std::sync::{Arc, RwLock};
use tracing::warn;

/// Trait for event handlers
///
/// Implement this trait for any handler that processes events of type E
///
/// # Example
///
/// ```
/// use allay_plugin::{Event, EventHandler};
/// use anyhow::Result;
/// use std::sync::Arc;
///
/// struct MyEvent {
///     pub message: String,
/// }
///
/// impl Event for MyEvent {}
///
/// struct MyEventHandler;
///
/// impl EventHandler<MyEvent> for MyEventHandler {
///     fn handle_event(self: Arc<Self>, event: Arc<MyEvent>) -> Result<()> {
///        println!("Handling event with message: {}", event.message);
///        Ok(())
///    }
/// }
/// ```
pub trait EventHandler<E: Event>: Send + Sync {
    fn handle_event(self: Arc<Self>, event: Arc<E>) -> anyhow::Result<()>;
}

/// Trait for asynchronous event handlers
///
/// Implement this trait for any handler that processes events of type E asynchronously
///
/// # Example
///
/// ```
/// use allay_plugin::{Event, AsyncEventHandler};
/// use anyhow::Result;
/// use std::sync::Arc;
/// use async_trait::async_trait;
///
/// struct MyEvent {
///    pub message: String,
/// }
///
/// impl Event for MyEvent {}
///
/// struct MyAsyncEventHandler;
///
/// #[async_trait]
/// impl AsyncEventHandler<MyEvent> for MyAsyncEventHandler {
///     async fn handle_event(self: Arc<Self>, event: Arc<MyEvent>) -> Result<()> {
///         println!("Handling async event with message: {}", event.message);
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait AsyncEventHandler<E: Event>: Send + Sync {
    async fn handle_event(self: Arc<Self>, event: Arc<E>) -> anyhow::Result<()>;
}

struct GenericEventBus<E: Event> {
    handlers: Vec<Arc<dyn EventHandler<E>>>,
    async_handlers: Vec<Arc<dyn AsyncEventHandler<E>>>,
}

impl<E: Event> GenericEventBus<E> {
    fn new() -> Self {
        Self {
            handlers: Vec::new(),
            async_handlers: Vec::new(),
        }
    }
}

/// Event bus to manage event handlers and publish events
pub struct EventBus(
    /// An [`Map`] to store the for different generic event types.
    RwLock<Map<dyn Any + Send + Sync>>,
);

impl Default for EventBus {
    fn default() -> Self {
        Self(RwLock::new(Map::new()))
    }
}

impl EventBus {
    /// Register an event handler for a specific event type E
    pub fn register_handler<H, E>(&self, handler: Arc<H>)
    where
        H: EventHandler<E> + 'static,
        E: Event + 'static,
    {
        let mut buses = self.0.write().unwrap();
        buses.entry().or_insert_with(GenericEventBus::new).handlers.push(handler);
    }

    /// Register an asynchronous event handler for a specific event type E
    pub fn register_async_handler<H, E>(&self, handler: Arc<H>)
    where
        H: AsyncEventHandler<E> + 'static,
        E: Event + 'static,
    {
        let mut buses = self.0.write().unwrap();
        buses.entry().or_insert_with(GenericEventBus::new).async_handlers.push(handler);
    }

    /// Publish an event to all registered handlers for the event type E
    pub async fn publish<E>(&self, event: Arc<E>)
    where
        E: Event + 'static,
    {
        // Copy the handlers to avoid holding the lock while process async operations
        let (sync_handlers, async_handlers) = self
            .0
            .read()
            .unwrap()
            .get::<GenericEventBus<E>>()
            .map(|bus| (bus.handlers.clone(), bus.async_handlers.clone()))
            .unwrap_or_default();

        // Handle sync handlers
        for handler in sync_handlers.into_iter() {
            if let Err(e) = handler.handle_event(event.clone()) {
                warn!("Error handling event: {}", e);
            }
        }

        // Handle async handlers
        let join_handlers = async_handlers
            .into_iter()
            .map(|handler| {
                let event = event.clone();

                tokio::spawn(async move {
                    if let Err(e) = handler.handle_event(event).await {
                        warn!("Error handling async event: {}", e);
                    }
                })
            })
            .collect::<Vec<_>>(); // collect to make sure all tasks are spawned before awaiting

        for handler in join_handlers {
            if let Err(e) = handler.await {
                warn!("Error awaiting async event handler: {}", e);
            }
        }
    }
}
