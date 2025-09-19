use allay_base::config::get_allay_config;

use crate::listener::{FileMapper, FilePublisher};

pub struct StaticPublisher;

impl FileMapper for StaticPublisher {
    fn src_root() -> String {
        get_allay_config().statics.dir.clone()
    }

    fn dest_root() -> String {
        get_allay_config().publish.dir.clone()
    }
}

impl FilePublisher for StaticPublisher {}
