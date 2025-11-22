use crate::route::utils::safe_filename;
use crate::route::{RouteError, RouteResult};
use axum::Json;
use axum::extract::State;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::UNIX_EPOCH;
use walkdir::WalkDir;

pub async fn handle_last_modify(
    State(root): State<Arc<PathBuf>>,
) -> RouteResult<Json<HashMap<String, u64>>> {
    last_modify(root).await.map(Json).ok_or(RouteError::InternalServerError(
        "Failed to get last modified times".into(),
    ))
}

pub async fn last_modify(root: Arc<PathBuf>) -> Option<HashMap<String, u64>> {
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
