mod content;
mod generator;
mod sitemap;

use content::ContentGeneratorWorker;
use sitemap::SiteMapWorker;
use std::sync::OnceLock;

/// Start the publishing workers.
pub fn start() {
    static CONTENT_WORKER: OnceLock<ContentGeneratorWorker> = OnceLock::new();
    CONTENT_WORKER.get_or_init(ContentGeneratorWorker::create).start();

    static SITEMAP_WORKER: OnceLock<SiteMapWorker> = OnceLock::new();
    SITEMAP_WORKER.get_or_init(|| SiteMapWorker).start();
}

/// Generate all files once.
pub fn generate_once() {
    ContentGeneratorWorker::create().generate_once();
}
