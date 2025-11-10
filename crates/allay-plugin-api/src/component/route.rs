use crate::{component::PluginGuest, plugin};
use exports::allay::plugin::route;
pub use route::{Error, Method, Response};

wit_bindgen::generate!({
   path: "wit/route.wit"
});

pub trait RouteComponent {
    fn handle_request(
        &self,
        _ty: Method,
        _url: String,
        _body: Option<Vec<u8>>,
    ) -> Option<Result<Response, Error>> {
        None
    }
}

impl route::Guest for PluginGuest {
    fn handle_request(
        ty: Method,
        url: String,
        body: Option<Vec<u8>>,
    ) -> Option<Result<Response, Error>> {
        plugin().route_component().handle_request(ty, url, body)
    }
}

export!(PluginGuest);
