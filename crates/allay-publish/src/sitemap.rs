use crate::generator::FileListener;
use allay_base::config::get_allay_config;
use allay_base::file::{self, FileResult};
use allay_base::sitemap::{SiteMap, UrlEntry};
use std::path::PathBuf;

/// A worker that manages the site map
pub struct SiteMapWorker;

impl FileListener for SiteMapWorker {
    fn root(&self) -> PathBuf {
        get_allay_config().content_dir.clone().into()
    }

    fn on_create(&self, path: PathBuf) -> FileResult<()> {
        let real_path = file::workspace(self.root().join(&path));
        let lastmod = file::last_modified(real_path)?;
        let mut map = SiteMap::write();
        let entry = UrlEntry { lastmod };
        map.urlset.insert(path, entry);
        map.dump();
        Ok(())
    }

    fn on_remove(&self, path: PathBuf) -> FileResult<()> {
        let mut map = SiteMap::write();
        map.urlset.remove(&path);
        map.dump();
        Ok(())
    }
}
