use crate::{component::PluginGuest, plugin_info};

wit_bindgen::generate!({
    path: "wit/base.wit"
});

impl Guest for PluginGuest {
    fn name() -> String {
        plugin_info().name.to_string()
    }

    fn version() -> String {
        plugin_info().version.to_string()
    }
}

export!(PluginGuest);
