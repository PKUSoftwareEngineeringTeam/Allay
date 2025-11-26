use crate::route::{RouteError, RouteResult};
use allay_base::config::get_theme_config;
use allay_base::file;
use allay_base::url::AllayUrlPath;
use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::task::spawn_blocking;

#[derive(Deserialize)]
pub struct LastModifiedParams {
    pub url: String,
}

pub async fn handle_last_modify(
    State(root): State<Arc<PathBuf>>,
    Query(params): Query<LastModifiedParams>,
) -> RouteResult<Json<u64>> {
    let path = root.join(&params.url);
    check_travesal(root.as_ref(), &path).await?;

    for path in AllayUrlPath::from(path).possible_paths() {
        if let Some(last_modify) = last_modify(path).await {
            return Ok(Json(last_modify));
        }
    }

    // return the 404 page if no file found
    let not_found = root.join(&get_theme_config().config.templates.not_found);
    last_modify(not_found).await.map(Json).ok_or(RouteError::Internal(
        "Failed to get last modified times".into(),
    ))
}

pub async fn check_travesal(root: &PathBuf, path: &PathBuf) -> RouteResult<()> {
    let canonical_root = fs::canonicalize(root)
        .await
        .map_err(|_| RouteError::Internal("Failed to canonicalize root directory".into()))?;
    let canonical_path = fs::canonicalize(&path)
        .await
        .map_err(|_| RouteError::Internal("Invalid path".into()))?;
    if !canonical_path.starts_with(&canonical_root) {
        return Err(RouteError::Internal("Path traversal detected".into()));
    }
    Ok(())
}

pub async fn last_modify(path: PathBuf) -> Option<u64> {
    spawn_blocking(move || file::last_modified(path)).await.ok()?.ok()
}
