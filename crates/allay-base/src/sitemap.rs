use crate::config::get_allay_config;
use crate::data::AllayData;
use crate::file;
use crate::log::NoPanicUnwrap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};
use tracing::warn;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SiteMap {
    #[serde(skip)]
    version: u32,
    pub urlset: HashMap<PathBuf, UrlEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UrlEntry {
    pub lastmod: u64,
    pub meta: AllayData,
}

static SITE_MAP: OnceLock<RwLock<SiteMap>> = OnceLock::new();

impl SiteMap {
    pub fn set_instance(instance: SiteMap) {
        SITE_MAP.set(RwLock::new(instance)).expect_("Site map instance is already set");
    }

    pub fn read() -> RwLockReadGuard<'static, Self> {
        SITE_MAP.wait().read().expect("Failed to acquire site map lock")
    }

    pub fn write() -> RwLockWriteGuard<'static, Self> {
        let mut write = SITE_MAP.wait().write().expect("Failed to acquire site map lock");
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
        if let Ok(content) = serde_json::to_string(&self)
            && file::write_file(&path, &content).is_ok()
        {
        } else {
            warn!("Failed to dump site map to file.");
        }
    }
}
