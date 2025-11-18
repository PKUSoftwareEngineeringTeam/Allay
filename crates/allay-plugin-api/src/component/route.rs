use super::PluginGuest;
use super::wit::route;
use crate::component::export::exports::allay::plugin::route::{Method, Request, Response};
use crate::http::unimplemented_response;
use crate::plugin;

/// Component trait for handling HTTP routes.
/// See the default implementation in [`unimplemented_response`]
pub trait RouteComponent: Sync + Send {
    /// Handle an incoming HTTP request and return a response.
    /// Both defined by crate [`axum`].
    fn handle(&self, _request: Request) -> Response {
        unimplemented_response()
    }

    /// Get the path of the route.
    fn route_paths(&self) -> Vec<(Method, String)> {
        Vec::new()
    }
}

impl route::Guest for PluginGuest {
    fn handle(request: Request) -> Response {
        plugin().route_component().handle(request)
    }

    fn route_paths() -> Vec<(Method, String)> {
        plugin().route_component().route_paths()
    }
}
