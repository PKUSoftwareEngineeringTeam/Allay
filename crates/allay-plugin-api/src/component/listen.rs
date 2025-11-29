use super::PluginGuest;
use super::wit::listen;
use crate::plugin;

pub trait ListenComponent {
    fn on_create(&self, _source: String) {}
    fn on_modify(&self, _source: String) {}
    fn on_remove(&self, _source: String) {}
}

impl listen::Guest for PluginGuest {
    fn on_create(source: String) {
        plugin().listen_component().on_create(source);
    }

    fn on_modify(source: String) {
        plugin().listen_component().on_modify(source);
    }

    fn on_remove(source: String) {
        plugin().listen_component().on_remove(source);
    }
}
