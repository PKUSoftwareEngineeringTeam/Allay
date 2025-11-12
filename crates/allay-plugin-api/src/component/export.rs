use super::PluginGuest;

wit_bindgen::generate!({ generate_all, skip: ["init-plugin"] });
export!(PluginGuest);
