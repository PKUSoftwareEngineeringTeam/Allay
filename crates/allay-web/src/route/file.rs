use crate::route::utils::safe_filename;
use crate::route::{RouteError, RouteResult};

use allay_base::config::get_theme_config;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, header, response::Builder};
use mime_guess::from_path;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

#[derive(Deserialize)]
pub struct DownloadParams {
    pub attachment: Option<bool>,
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

pub async fn handle_index(
    State(root): State<Arc<PathBuf>>,
    Query(params): Query<DownloadParams>,
) -> RouteResult {
    let index = &get_theme_config().config.templates.index;
    file_response(&index.into(), &params, root).await
}

pub async fn file_response(
    file_path: &PathBuf,
    params: &DownloadParams,
    root: Arc<PathBuf>,
) -> RouteResult {
    let path = root.join(file_path);

    // check whether path is a file
    if !path.exists() || !path.is_file() {
        return Err(RouteError::NotFound);
    }
    if path.strip_prefix(root.as_ref()).is_err() {
        return Err(RouteError::Forbidden);
    }

    let metadata = tokio::fs::metadata(&path)
        .await
        .map_err(|e| RouteError::InternalServerError(format!("Failed to get metadata: {}", e)))?;

    let file = File::open(&path)
        .await
        .map_err(|e| RouteError::InternalServerError(format!("Failed to open file: {}", e)))?;

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

pub async fn handle_file(
    State(root): State<Arc<PathBuf>>,
    Path(file_path): Path<String>,
    Query(params): Query<DownloadParams>,
) -> RouteResult {
    // foo.html -> foo.html
    let path = PathBuf::from(&file_path);
    // foo/ -> foo/index.html
    let sub_index = path.join(&get_theme_config().config.templates.index);
    // foo -> foo.html
    let html_file = path.with_extension("html");

    let mut possible_paths = vec![&path];
    if file_path.ends_with("/") {
        possible_paths.push(&sub_index);
    } else if path.extension().is_none() {
        possible_paths.push(&html_file);
    }

    for path in possible_paths.into_iter() {
        let response = file_response(path, &params, root.clone()).await;
        if let Err(RouteError::NotFound) = response {
            continue; // try next possible path
        }
        return response;
    }

    let not_found = &get_theme_config().config.templates.not_found;
    file_response(&not_found.into(), &params, root).await
}
