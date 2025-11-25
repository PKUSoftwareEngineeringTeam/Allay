use crate::route::{RouteError, RouteResult};
use allay_base::config::get_theme_config;
use allay_base::url::AllayUrlPath;
use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::UNIX_EPOCH;

#[derive(Deserialize)]
pub struct LastModifiedParams {
    pub url: String,
}

pub async fn handle_last_modify(
    State(root): State<Arc<PathBuf>>,
    Query(params): Query<LastModifiedParams>,
) -> RouteResult<Json<u64>> {
    let path = root.join(&params.url);
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

pub async fn last_modify(path: PathBuf) -> Option<u64> {
    let metadata = tokio::fs::metadata(path).await.ok()?;
    let modified_time = metadata.modified().ok()?;
    let duration = modified_time.duration_since(UNIX_EPOCH).ok()?;
    Some(duration.as_secs())
}
