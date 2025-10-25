use crate::routes::RouteEvent;
use allay_base::config::get_allay_config;
use allay_plugin::{EventHandler, Plugin, PluginContext};
use async_trait::async_trait;
use axum::Json;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, Response, StatusCode, header, response::Builder};
use axum::routing::get;
use mime_guess::from_path;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{self, PathBuf};
use std::sync::Arc;
use std::time::UNIX_EPOCH;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use walkdir::WalkDir;

fn safe_filename(file_path: &str) -> String {
    path::Path::new(file_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .replace('"', "")
}

fn force_download(mime_type: &str) -> bool {
    matches!(
        mime_type,
        "application/zip"
            | "application/pdf"
            | "application/octet-stream"
            | "application/x-rar-compressed"
    )
}

#[derive(Deserialize)]
struct DownloadParams {
    pub attachment: Option<bool>,
}

async fn file_response(
    file_path: &str,
    params: &DownloadParams,
    root: Arc<PathBuf>,
) -> Result<Response<Body>, (StatusCode, String)> {
    let path = root.join(file_path);
    // check whether path is a file
    if !path.exists() || !path.is_file() {
        return Err((StatusCode::NOT_FOUND, "Not Found".to_string()));
    }
    if path.strip_prefix(root.as_ref()).is_err() {
        return Err((StatusCode::FORBIDDEN, "Forbidden".to_string()));
    }

    let metadata = tokio::fs::metadata(&path)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let file = File::open(&path)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let mime_type = from_path(&path).first_or_octet_stream();

    let content_disposition =
        if params.attachment.unwrap_or(false) || force_download(mime_type.as_ref()) {
            format!("attachment; filename=\"{}\"", safe_filename(file_path))
        } else {
            format!("inline; filename=\"{}\"", safe_filename(file_path))
        };

    let response = Builder::new()
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_str(mime_type.as_ref()).unwrap(),
        )
        .header(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_str(&content_disposition).unwrap(),
        )
        .header(header::CONTENT_LENGTH, HeaderValue::from(metadata.len()))
        .header(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=3600"),
        )
        .body(body)
        .unwrap();

    Ok(response)
}

async fn handle_file(
    State(root): State<Arc<PathBuf>>,
    Path(file_path): Path<String>,
    Query(params): Query<DownloadParams>,
) -> Result<Response<Body>, (StatusCode, String)> {
    match file_response(&file_path, &params, Arc::clone(&root)).await {
        Ok(response) => Ok(response),
        Err((StatusCode::NOT_FOUND, _)) => {
            file_response(&get_allay_config().theme.template.not_found, &params, root).await
        }
        Err(err) => Err(err),
    }
}

async fn handle_index(
    State(root): State<Arc<PathBuf>>,
    Query(params): Query<DownloadParams>,
) -> Result<Response<Body>, (StatusCode, String)> {
    file_response(&get_allay_config().theme.template.index, &params, root).await
}

async fn handle_last_modify(
    State(root): State<Arc<PathBuf>>,
) -> Result<Json<HashMap<String, u64>>, (StatusCode, String)> {
    match last_modify(root).await {
        Some(files) => Ok(Json(files)),
        None => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )),
    }
}

async fn last_modify(root: Arc<PathBuf>) -> Option<HashMap<String, u64>> {
    let mut files = HashMap::new();

    // travel through all file in `root` and get their last modified times
    for entry in WalkDir::new(root.as_ref()).into_iter().filter_map(|x| x.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();

            let metadata = tokio::fs::metadata(path).await.ok()?;
            let modified_time = metadata.modified().ok()?;
            files.insert(
                safe_filename(path.file_name().map(|s| s.to_str())??),
                modified_time.duration_since(UNIX_EPOCH).ok()?.as_secs(),
            );
        }
    }

    Some(files)
}

pub struct BuiltinRouteHandler;

#[async_trait]
impl EventHandler<RouteEvent> for BuiltinRouteHandler {
    async fn handle_event(&self, event: &mut RouteEvent) -> anyhow::Result<()> {
        event.register("/api/last-modified", get(handle_last_modify));
        event.register("/{*path}", get(handle_file));
        event.register("/", get(handle_index));
        Ok(())
    }
}

pub struct BuiltinRoutePlugin;

#[async_trait]
impl Plugin for BuiltinRoutePlugin {
    fn name(&self) -> &str {
        "builtin-route-plugin"
    }

    async fn initialize(&self, context: PluginContext) -> anyhow::Result<()> {
        let handler = Arc::new(BuiltinRouteHandler);
        context.event_bus.register_handler(handler).await;
        Ok(())
    }
}
