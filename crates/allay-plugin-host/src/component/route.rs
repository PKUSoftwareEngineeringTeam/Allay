use super::wit::route;
use crate::PluginHost;
use allay_base::async_util::{AsyncFrom, AsyncInto};
use axum::body::{Body, to_bytes};
use axum::extract::Request;
use axum::http::{Method, StatusCode};
use axum::response::Response;
use wasmtime::AsContextMut;

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

#[async_trait::async_trait]
impl AsyncFrom<Request> for route::Request {
    async fn async_from(request: Request) -> Self {
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
    pub async fn handle_request(&self, request: Request) -> wasmtime::Result<Response> {
        let plugin = self.plugin.clone();
        let mut store = self.store.lock().await;

        let response = self
            .instance
            .run_concurrent(store.as_context_mut(), async move |accessor| {
                plugin
                    .allay_plugin_route()
                    .call_handle(accessor, request.async_into().await)
                    .await
            })
            .await??;

        Ok(response.into())
    }

    pub fn route_path(&self) -> wasmtime::Result<Vec<(Method, String)>> {
        let mut store = self.store.blocking_lock();
        let path = self
            .plugin
            .allay_plugin_route()
            .call_route_path(store.as_context_mut())?
            .into_iter()
            .map(|(method, path)| (Method::from(method), path))
            .collect();

        Ok(path)
    }
}
