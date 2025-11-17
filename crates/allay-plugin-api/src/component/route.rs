use super::PluginGuest;
use super::wit::route;
use crate::plugin;
use axum::body::{Body, to_bytes};
use axum::extract::Request;
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use tokio::runtime;

/// Component trait for handling HTTP routes.
/// See the default implementation in [`unimplemented_response`]
pub trait RouteComponent: Sync + Send {
    /// Handle an incoming HTTP request and return a response.
    /// Both defined by crate [`axum`].
    fn handle(&self, _request: Request) -> Response {
        unimplemented_response()
    }

    /// Get the path of the route.
    fn route_path(&self) -> Vec<(route::Method, String)> {
        Vec::new()
    }
}

/// A helpers trait for RouteComponent that allows returning errors.
///
/// Note: The trait is automatically implemented for [`RouteComponent`]
pub trait TryRouteComponent: Sync + Send + RouteComponent {
    /// Error type for the route handler
    type Error: IntoResponse;

    fn try_handle(&self, _request: Request) -> Result<Response, Self::Error> {
        Ok(unimplemented_response())
    }
}

impl<T: TryRouteComponent> RouteComponent for T {
    fn handle(&self, request: Request) -> Response {
        self.try_handle(request).into_response()
    }
}

/// Default response for unimplemented routes
/// Returns a 501 Not Implemented response
pub fn unimplemented_response() -> Response {
    Response::builder()
        .status(StatusCode::NOT_IMPLEMENTED)
        .body(Body::empty())
        .unwrap()
}

impl From<route::Method> for Method {
    fn from(method: route::Method) -> Method {
        match method {
            route::Method::Get => Method::GET,
            route::Method::Post => Method::POST,
            route::Method::Put => Method::PUT,
            route::Method::Delete => Method::DELETE,
        }
    }
}

impl route::Guest for PluginGuest {
    fn handle(request: route::Request) -> route::Response {
        plugin().route_component().handle(request.into()).into()
    }
    fn route_paths() -> Vec<(route::Method, String)> {
        plugin().route_component().route_path()
    }
}

impl From<route::Request> for Request {
    fn from(request: route::Request) -> Request {
        let mut builder = Request::builder()
            .method(request.ty) // Set method
            .uri(request.uri); // Set URI

        // Set headers
        for (name, value) in request.headers {
            builder = builder.header(name, value);
        }

        // Set body
        let body = Body::from(request.body);

        builder.body(body).unwrap_or_default()
    }
}

impl From<Response> for route::Response {
    fn from(response: Response) -> route::Response {
        // Extract status code
        let status_code = response.status().as_u16();

        // Convert headers
        let mut headers = Vec::new();

        for (name, value) in response.headers() {
            headers.push((name.as_str().into(), value.as_bytes().into()));
        }

        // Extract body4
        let runtime = runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let body = runtime
            .block_on(to_bytes(response.into_body(), usize::MAX))
            .unwrap_or_default()
            .to_vec();

        route::Response {
            status_code,
            headers,
            body,
        }
    }
}
