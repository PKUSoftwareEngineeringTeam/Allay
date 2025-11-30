mod content;
mod generator;
#[cfg(feature = "plugin")]
mod plugin;
mod process;
mod sitemap;

use content::ContentGeneratorWorker;
use generator::FileListener;
#[cfg(feature = "plugin")]
use plugin::PluginListener;
use sitemap::SiteMapWorker;
use std::sync::OnceLock;

/// Start the publishing workers.
pub fn start() {
    static CONTENT_WORKER: OnceLock<ContentGeneratorWorker> = OnceLock::new();
    CONTENT_WORKER.get_or_init(ContentGeneratorWorker::create).start();

    static SITEMAP_WORKER: OnceLock<SiteMapWorker> = OnceLock::new();
    SITEMAP_WORKER.get_or_init(SiteMapWorker::create).start_listening();

    #[cfg(feature = "plugin")]
    static PLUGIN_WORKER: OnceLock<PluginListener> = OnceLock::new();
    #[cfg(feature = "plugin")]
    PLUGIN_WORKER.get_or_init(|| PluginListener).start_listening();
}

/// Generate all files once.
pub fn generate_once() {
    ContentGeneratorWorker::create().generate_once();
    SiteMapWorker.cold_start();
    #[cfg(feature = "plugin")]
    PluginListener.cold_start();
}
