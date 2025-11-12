use super::PluginGuest;
use super::wit::route;
use crate::plugin;
use axum::body::{Body, to_bytes};
use axum::extract::Request;
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};

/// Component trait for handling HTTP routes.
/// See the default implementation in [`unimplemented_response`]
#[async_trait::async_trait]
pub trait RouteComponent: Sync + Send {
    /// Handle an incoming HTTP request and return a response.
    /// Both defined by crate [`axum`].
    async fn handle(&self, _request: Request) -> Response {
        unimplemented_response()
    }
}

/// A helpers trait for RouteComponent that allows returning errors.
///
/// Note: The trait is automatically implemented for [`RouteComponent`]
#[async_trait::async_trait]
pub trait TryRouteComponent: Sync + Send + RouteComponent {
    /// Error type for the route handler
    type Error: IntoResponse;

    async fn try_handle(&self, _request: Request) -> Result<Response, Self::Error> {
        Ok(unimplemented_response())
    }
}

#[async_trait::async_trait]
impl<T: TryRouteComponent> RouteComponent for T {
    async fn handle(&self, request: Request) -> Response {
        self.try_handle(request).await.into_response()
    }
}

#[async_trait::async_trait]
pub trait AsyncInto<T> {
    async fn async_into(self) -> T;
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
    async fn handle(request: route::Request) -> route::Response {
        plugin().route_component().handle(request.into()).await.async_into().await
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
        let body = match request.body {
            Some(bytes) => Body::from(bytes),
            None => Body::empty(),
        };

        builder.body(body).unwrap_or_default()
    }
}

#[async_trait::async_trait]
impl AsyncInto<route::Response> for Response {
    async fn async_into(self) -> route::Response {
        // Extract status code
        let status_code = self.status().as_u16();

        // Convert headers
        let mut headers = Vec::new();

        for (name, value) in self.headers() {
            headers.push((name.as_str().into(), value.as_bytes().into()));
        }

        let body = to_bytes(self.into_body(), usize::MAX).await.unwrap_or_default();
        let body = (!body.is_empty()).then_some(body.to_vec());

        route::Response {
            status_code,
            headers,
            body,
        }
    }
}
