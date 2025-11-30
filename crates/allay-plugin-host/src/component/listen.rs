use crate::{PluginHost, WasmOk};

impl PluginHost {
    pub fn on_create(&mut self, source: String) {
        self.plugin.allay_plugin_listen().call_on_create(&mut self.store, &source).ok_();
    }

    pub fn on_modify(&mut self, source: String) {
        self.plugin.allay_plugin_listen().call_on_modify(&mut self.store, &source).ok_();
    }

    pub fn on_remove(&mut self, source: String) {
        self.plugin.allay_plugin_listen().call_on_remove(&mut self.store, &source).ok_();
    }
}
