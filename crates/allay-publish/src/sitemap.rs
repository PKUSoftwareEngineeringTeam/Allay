use crate::generator::FileListener;
use allay_base::config::get_allay_config;
use allay_base::file::{self, FileResult};
use allay_base::sitemap::{SiteMap, UrlEntry};
use allay_compiler::get_meta;
use std::ops::DerefMut;
use std::path::PathBuf;
use tracing::warn;

/// A worker that manages the site map
pub struct SiteMapWorker;

impl SiteMapWorker {
    pub fn create() -> Self {
        let instance = SiteMapWorker;
        let mut map = SiteMap::default();

        let root = file::workspace(instance.root());

        // scan all files in the content directory
        let dir = file::read_dir_all_files(&root).expect("content dir does not exit");

        for file in dir {
            let path = file.strip_prefix(&root).unwrap().into();
            if let Err(e) = instance.create_on(path, &mut map) {
                warn!("Failed to add file to sitemap: {}", e);
            }
        }
        SiteMap::set_instance(map);

        instance
    }

    fn create_on(&self, path: PathBuf, map: &mut SiteMap) -> FileResult<()> {
        let real_path = file::workspace(self.root().join(&path));
        let lastmod = file::last_modified(&real_path)?;
        let meta = get_meta(real_path).unwrap_or_default().into();
        let entry = UrlEntry { lastmod, meta };
        map.urlset.insert(path, entry);
        Ok(())
    }
}

impl FileListener for SiteMapWorker {
    fn root(&self) -> PathBuf {
        get_allay_config().content_dir.clone().into()
    }

    fn on_create(&self, path: PathBuf) -> FileResult<()> {
        let mut map = SiteMap::write();
        self.create_on(path, map.deref_mut())?;
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
