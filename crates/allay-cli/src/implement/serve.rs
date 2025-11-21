use allay_base::config::theme::get_theme_config;
use allay_base::config::{ServeArgs, get_allay_config};
use allay_base::file;
use allay_base::log::show_error;
#[cfg(feature = "plugin")]
use allay_plugin::PluginManager;
use allay_web::server::Server;
use tracing::instrument;

/// CLI Server Command
#[instrument(name = "serving the site", skip(args))]
pub fn serve(args: &ServeArgs) -> anyhow::Result<()> {
    let url = format!("http://{}:{}", args.address, args.port);

    println!("Starting the site server at {}", url);
    allay_publish::start();

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

    if args.open {
        webbrowser::open(&url).unwrap_or_else(|_| println!("Failed to open the browser"));
    }

    let server = Server::new(
        file::workspace(get_allay_config().publish_dir.as_str()),
        args.port,
        args.address.clone(),
    );
    server.serve()?;
    Ok(())
}
