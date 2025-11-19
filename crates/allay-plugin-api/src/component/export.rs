use super::PluginGuest;

pub use exports::allay::plugin::route::{Header, Method, Request, Response};

wit_bindgen::generate!({ generate_all, skip: ["init-plugin"] });
export!(PluginGuest);
