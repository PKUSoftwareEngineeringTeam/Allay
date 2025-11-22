mod file;
mod hot_reload;
mod utils;

#[cfg(feature = "plugin")]
use crate::plugin_worker::PluginWorker;
use crate::route::file::{handle_file, handle_index};
use crate::route::hot_reload::handle_last_modify;
#[cfg(feature = "plugin")]
use allay_plugin::PluginManager;
#[cfg(feature = "plugin")]
use allay_plugin::manager::Plugin;
use axum::Router;
#[cfg(feature = "plugin")]
use axum::http::Method;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
#[cfg(feature = "plugin")]
use axum::routing::{delete, post, put};
use std::path::PathBuf;
use std::sync::Arc;
#[cfg(feature = "plugin")]
use std::sync::LazyLock;

enum RouteError {
    NotFound,
    Forbidden,
    InternalServerError(String),
}

impl IntoResponse for RouteError {
    fn into_response(self) -> Response {
        match self {
            RouteError::NotFound => (StatusCode::NOT_FOUND, "Not Found").into_response(),
            RouteError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden").into_response(),
            RouteError::InternalServerError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg).into_response()
            }
        }
    }
}

type RouteResult<T = Response> = Result<T, RouteError>;

#[cfg(feature = "plugin")]
fn plugin_worker() -> &'static PluginWorker {
    static PLUGIN_WORKER: LazyLock<PluginWorker> = LazyLock::new(|| PluginWorker::new(2));
    &PLUGIN_WORKER
}

#[cfg(feature = "plugin")]
fn register_custom_route(router: Router, plugin: Plugin) -> Router {
    let mut plugin_host = plugin.lock().expect("poisoned lock");
    let route_paths = plugin_host.route_paths();
    drop(plugin_host);

    if let Ok(route_path) = route_paths {
        route_path.into_iter().fold(router, |router, (method, path)| {
            let plugin = plugin.clone();
            let handler = async move |req| -> Response {
                let req = allay_plugin::types::Request::from_axum(req).await;
                plugin_worker().handle_request(plugin, req).into()
            };
            match method {
                Method::GET => router.route(&path, get(handler)),
                Method::POST => router.route(&path, post(handler)),
                Method::PUT => router.route(&path, put(handler)),
                Method::DELETE => router.route(&path, delete(handler)),
                _ => router,
            }
        })
    } else {
        router
    }
}

pub fn build_route(path: PathBuf) -> Router {
    let route = Router::new()
        .route("/api/last-modified", get(handle_last_modify))
        .route("/{*path}", get(handle_file))
        .route("/", get(handle_index))
        .with_state(Arc::new(path));
    cfg_if::cfg_if! {
        if #[cfg(feature = "plugin")] {
            let plugin_manager = PluginManager::instance();
            plugin_manager.plugins().into_iter().fold(route, register_custom_route)
        } else {
            route
        }
    }
}
