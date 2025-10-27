mod server;

pub use server::*;

/// Marker trait for events
/// 
/// Implement this trait for any event type you want to use with the EventBus
pub trait Event: Sync + Send {}
