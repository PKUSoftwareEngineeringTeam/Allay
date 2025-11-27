use crate::config::get_allay_config;
use crate::file;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{LazyLock, RwLock, RwLockReadGuard, RwLockWriteGuard};
use tracing::warn;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SiteMap {
    version: u32,
    pub urlset: HashMap<PathBuf, UrlEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UrlEntry {
    pub lastmod: u64,
}

static SITE_MAP: LazyLock<RwLock<SiteMap>> = LazyLock::new(|| RwLock::new(SiteMap::default()));

impl SiteMap {
    pub fn read() -> RwLockReadGuard<'static, Self> {
        SITE_MAP.read().expect("Failed to acquire site map lock")
    }

    pub fn write() -> RwLockWriteGuard<'static, Self> {
        let mut write = SITE_MAP.write().expect("Failed to acquire site map lock");
        write.version += 1;
        write
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    fn filepath() -> PathBuf {
        const FILE: &str = "sitemap.json";
        file::workspace(&get_allay_config().publish_dir).join(FILE)
    }

    pub fn dump(&self) {
        let path = Self::filepath();
        if let Ok(content) = serde_json::to_string_pretty(self)
            && file::write_file(&path, &content).is_ok()
        {
        } else {
            warn!("Failed to dump site map to file.");
        }
    }
}
