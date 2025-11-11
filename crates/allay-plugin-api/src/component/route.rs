use super::PluginGuest;
use super::wit::route;
use crate::plugin;
use axum::body::{Body, to_bytes};
use axum::extract::Request;
use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::response::Response;

#[async_trait::async_trait]
pub trait RouteComponent: Sync + Send {
    async fn handle(&self, _request: Request) -> Response {
        unimplemented_response()
    }
}

pub fn unimplemented_response() -> Response {
    Response::builder()
        .status(StatusCode::NOT_IMPLEMENTED)
        .body(Body::empty())
        .unwrap()
}

impl route::Guest for PluginGuest {
    async fn handle(request: route::Request) -> route::Response {
        let axum_request = wit_to_axum_request(request);
        let axum_response = plugin().route_component().handle(axum_request).await;
        axum_to_wit_response(axum_response).await
    }
}

/// Convert WIT request to axum Request
fn wit_to_axum_request(wit_request: route::Request) -> Request {
    let mut builder = Request::builder();

    // Set method
    builder = match wit_request.ty {
        route::Method::Get => builder.method("GET"),
        route::Method::Post => builder.method("POST"),
        route::Method::Put => builder.method("PUT"),
        route::Method::Delete => builder.method("DELETE"),
    };

    // Set URI
    builder = builder.uri(&wit_request.uri);

    // Set headers
    for (name, value) in wit_request.headers {
        if let Ok(name) = HeaderName::from_bytes(name.as_bytes())
            && let Ok(value) = HeaderValue::from_str(&value)
        {
            builder = builder.header(name, value);
        }
    }

    // Set body
    let body = match wit_request.body {
        Some(bytes) => Body::from(bytes),
        None => Body::empty(),
    };

    builder.body(body).unwrap_or_default()
}

/// Convert axum Response to WIT response
async fn axum_to_wit_response(axum_response: Response) -> route::Response {
    // Extract status code
    let status_code = axum_response.status().as_u16();

    // Convert headers
    let mut headers = Vec::new();
    for (name, value) in axum_response.headers() {
        if let Ok(value) = value.to_str() {
            headers.push((name.as_str().into(), value.into()));
        }
    }

    let body = to_bytes(axum_response.into_body(), usize::MAX).await.unwrap_or_default();
    let body = (!body.is_empty()).then_some(body.to_vec());

    route::Response {
        status_code,
        headers,
        body,
    }
}
