use allay_base::config::{BuildArgs, get_theme_config};
use allay_base::log::show_error;
#[cfg(feature = "plugin")]
use allay_plugin::PluginManager;
use anyhow::Ok;
use tracing::instrument;

pub fn load_plugins() -> anyhow::Result<()> {
    // load plugins
    cfg_if::cfg_if! {
        if #[cfg(feature = "plugin")] {
            allay_plugin::load_plugins();
            let plugin_names = PluginManager::instance().plugin_names();
            if !plugin_names.is_empty() {
                println!("Loaded plugins: {}", plugin_names.join(", "));
            }
        }
    }

    // check plugin dependencies
    cfg_if::cfg_if! {
        if #[cfg(feature = "plugin")] {
            let plugin_manager = PluginManager::instance();
            let required_plugins = &get_theme_config().dependencies.plugins;
            for (name, version) in required_plugins {
                if !plugin_manager.version_match(name, version)? {
                    show_error(&format!("Plugin {} version {} is required", name, version));
                }
            }
        } else {
            if !get_theme_config().dependencies.plugins.is_empty() {
                show_error("Plugin dependencies are not supported without the plugin feature");
            }
        }
    }

    Ok(())
}

/// CLI Build Command
#[instrument(name = "building the site", skip(_args))]
pub fn build(_args: &BuildArgs) -> anyhow::Result<()> {
    load_plugins()?;
    allay_publish::generate_once();
    Ok(())
}
