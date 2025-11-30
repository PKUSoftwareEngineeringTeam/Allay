use crate::generator::FileListener;
use allay_base::config::{CLICommand, get_allay_config, get_cli_config, get_site_config};
use allay_base::file::{self, FileResult};
use allay_base::log::NoPanicUnwrap;
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

        map.base_url = if get_cli_config().online {
            // In online mode, use the base_url from site config
            get_site_config()
                .get("base_url")
                .expect_("base_url not found in online mode")
                .as_str()
                .expect_("base_url should be a string")
                .clone()
        } else if let CLICommand::Serve(args) = &get_cli_config().command {
            // In serve mode, use the local address and port
            format!("http://{}:{}/", args.address, args.port)
        } else {
            String::new()
        };

        let root = file::workspace(instance.root());

        // scan all files in the content directory
        let dir = file::read_dir_all_files(&root).expect_("content dir does not exit");

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
