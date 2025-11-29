//! APIs for Allay Plugins.

mod component;
pub mod http;

pub use component::*;
use std::sync::OnceLock;

/// Trait for Allay Plugins. You must implement this trait for your plugin struct.
/// You also need to register your plugin using the `register_plugin!` macro.
///
/// # Example
///
/// ```
/// use allay_plugin_api as allay;
///
/// struct MyPlugin;
///
/// impl allay::Plugin for MyPlugin {
///     fn name() -> &'static str where Self: Sized {
///         "my-plugin"
///     }
///     fn version() -> &'static str where Self: Sized {
///         "0.1.0"
///     }
///     fn new() -> Self where Self: Sized {
///         MyPlugin
///     }
/// }
///
/// allay::register_plugin!(MyPlugin);
/// ```
#[plugin_macro::components(CompilerComponent, RouteComponent, ListenComponent)]
pub trait Plugin: Sync + Send {
    /// Name of the plugin.
    fn name() -> &'static str
    where
        Self: Sized;

    /// Version of the plugin.
    fn version() -> &'static str
    where
        Self: Sized;

    /// Create a new instance of the plugin.
    fn new() -> Self
    where
        Self: Sized;
}

struct PluginInfo {
    name: &'static str,
    version: &'static str,
}

static PLUGIN: OnceLock<Box<dyn Plugin>> = OnceLock::new();
static PLUGIN_INFO: OnceLock<PluginInfo> = OnceLock::new();

pub fn register_plugin<T: Plugin + 'static>() {
    let name = T::name();
    let version = T::version();
    PLUGIN_INFO.get_or_init(|| PluginInfo { name, version });
    PLUGIN.get_or_init(|| Box::new(T::new()));
}

fn plugin() -> &'static dyn Plugin {
    PLUGIN.get_or_init(|| panic!("Plugin not registered")).as_ref()
}

fn plugin_info() -> &'static PluginInfo {
    PLUGIN_INFO.get_or_init(|| panic!("Plugin not registered"))
}

/// Macro to register your Allay plugin. You must call this macro with your plugin struct type.
/// Otherwise, the plugin will not work.
#[macro_export]
macro_rules! register_plugin {
    ($plugin: ty) => {
        #[unsafe(export_name = "init-plugin")]
        pub fn __register_plugin() {
            allay_plugin_api::register_plugin::<$plugin>();
        }
    };
}
