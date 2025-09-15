use crate::listener::{FileMapper, FilePublisher};
use allay_base::config::ALLAY_CONFIG;

pub struct StaticPublisher;

impl FileMapper for StaticPublisher {
    fn src_root() -> String {
        ALLAY_CONFIG.statics.dir.clone()
    }

    fn dest_root() -> String {
        ALLAY_CONFIG.publish.dir.clone()
    }
}

impl FilePublisher for StaticPublisher {}
