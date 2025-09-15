use crate::listener::FileMapper;
use crate::publish::StaticPublisher;

pub mod listener;
pub mod publish;

pub fn start() {
    // make clippy happy
    StaticPublisher::src_root();
}
