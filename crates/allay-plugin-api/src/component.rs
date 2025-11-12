pub mod compiler;
mod export;
pub mod route;

use crate::plugin_info;
pub use compiler::CompilerComponent;
use export::exports::allay::plugin as wit;
pub use route::RouteComponent;

struct PluginGuest;

impl export::Guest for PluginGuest {
    fn name() -> String {
        plugin_info().name.to_string()
    }

    fn version() -> String {
        plugin_info().version.to_string()
    }
}
