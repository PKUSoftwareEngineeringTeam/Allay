use super::wit::route;
use crate::PluginHost;
use axum::body::{Body, to_bytes};
use axum::extract::Request;
use axum::http::{Method, StatusCode};
use axum::response::Response;

impl From<route::Method> for Method {
    fn from(method: route::Method) -> Self {
        match method {
            route::Method::Get => Method::GET,
            route::Method::Post => Method::POST,
            route::Method::Put => Method::PUT,
            route::Method::Delete => Method::DELETE,
        }
    }
}

impl route::Request {
    pub async fn from_axum(request: Request) -> Self {
        let (parts, body) = request.into_parts();

        // Convert method
        let ty = match parts.method {
            Method::GET => route::Method::Get,
            Method::POST => route::Method::Post,
            Method::PUT => route::Method::Put,
            Method::DELETE => route::Method::Delete,
            _ => route::Method::Get, // Default to Get for unknown methods
        };

        // Convert URI
        let uri = parts.uri.to_string();

        // Convert headers
        let mut headers = Vec::new();
        for (name, value) in parts.headers {
            if let Some(name_str) = name {
                headers.push((name_str.to_string(), value.as_bytes().to_vec()));
            }
        }

        let body = to_bytes(body, usize::MAX).await.unwrap_or_default().to_vec();

        route::Request {
            ty,
            uri,
            headers,
            body,
        }
    }
}

impl From<route::Response> for Response {
    fn from(response: route::Response) -> Self {
        let mut builder = Response::builder().status(response.status_code);

        // Set headers
        for (name, value) in response.headers {
            builder = builder.header(name, value);
        }

        // Set body
        let body = Body::from(response.body);

        builder.body(body).unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap()
        })
    }
}

impl PluginHost {
    pub fn handle_request(&mut self, request: route::Request) -> wasmtime::Result<route::Response> {
        self.plugin.allay_plugin_route().call_handle(&mut self.store, &request)
    }

    pub fn route_paths(&mut self) -> wasmtime::Result<Vec<(Method, String)>> {
        let path = self
            .plugin
            .allay_plugin_route()
            .call_route_paths(&mut self.store)?
            .into_iter()
            .map(|(method, path)| (method.into(), path))
            .collect();

        Ok(path)
    }
}
