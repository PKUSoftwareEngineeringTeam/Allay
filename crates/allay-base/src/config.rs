pub mod cli;
pub mod site;

use crate::costants::CONFIG_FILE;
use crate::data::{AllayData, AllayObject};
use crate::file::FileUtils;
pub use cli::{AllayCLI, Parser};
use std::sync::OnceLock;

#[derive(Debug)]
struct Configs {
    cli: AllayCLI,
    site: AllayObject,
}

static GLOBAL_CONFIG: OnceLock<Configs> = OnceLock::new();

pub struct GlobalConfigs;

impl GlobalConfigs {
    pub fn init() -> Result<(), String> {
        let cli = AllayCLI::parse();

        let site = &FileUtils::read_file(FileUtils::absolute_path(CONFIG_FILE).unwrap())
            .map_err(|err| err.to_string())?
            .content;
        let site =
            AllayData::from_toml(site).map_err(|err| format!("Parse config error: {}", err))?;

        GLOBAL_CONFIG
            .set(Configs { cli, site })
            .map_err(|_| "Global configs already initialized".into())
    }

    fn get() -> &'static Configs {
        GLOBAL_CONFIG.get().expect("Global configs not initialized")
    }

    pub fn cli() -> &'static AllayCLI {
        &Self::get().cli
    }

    pub fn site() -> &'static AllayObject {
        &Self::get().site
    }
}
