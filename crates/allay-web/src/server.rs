//! A simple HTTP server.

use crate::ServerResult;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, Response, StatusCode, header, response::Builder};
use axum::routing::get;
use axum::{Json, Router};
use mime_guess::from_path;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{self, PathBuf};
use std::sync::Arc;
use std::time::UNIX_EPOCH;
use tokio::fs::File;
use tokio::net::TcpListener;
use tokio::runtime;
use tokio_util::io::ReaderStream;
use walkdir::WalkDir;

#[derive(Deserialize)]
struct DownloadParams {
    attachment: Option<bool>,
}

/// Represents a server configuration.
///
/// The `Server` struct holds the necessary information to configure and
/// identify a server, including its file path, port number, and host address.
///
/// You can use [Server::serve] to start the server,
pub struct Server {
    path: PathBuf,
    port: u16,
    host: String,
}

impl Server {
    /// Creates a new `Server` instance.
    ///
    /// # Arguments
    /// - `path` - A reference to the path of the server's directory. This can be any type that can be referenced as a `std::path::Path`.
    /// - `port` - The port number on which the server will listen for incoming connections. Must be a 16-bit unsigned integer.
    /// - `host` - The hostname or IP address from which the server will accept connections. This should be provided as a `String`.
    ///
    /// # Returns
    /// A new `Server` instance configured with the provided path, port, and host.
    ///
    /// # Examples
    /// ```
    /// use allay_web::server::Server;
    /// let server = Server::new("/path/to/directory", 8080, "localhost".to_string());
    /// ```
    pub fn new<P: AsRef<path::Path>>(path: P, port: u16, host: String) -> Self {
        Server {
            path: path.as_ref().into(),
            port,
            host,
        }
    }

    /// Starts the server to serve files from the specified path. This will block the current thread
    ///
    /// # Returns
    ///
    /// * `ServerResult<()>` - A result that indicates whether the operation was successful or an error occurred.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The runtime cannot be built.
    /// - Binding to the specified address fails.
    /// - There is an error in serving the application.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use allay_web::server::Server;
    /// let server = Server::new("/path/to/directory", 8080, "localhost".to_string());
    /// server.serve().unwrap();
    /// ```
    pub fn serve(&self) -> ServerResult<()> {
        let addr = format!("{}:{}", self.host, self.port);
        let app = Router::new()
            .route("/api/last-modified", get(Self::handle_last_modify))
            .route("/{*path}", get(Self::handle_file))
            .with_state(Arc::new(self.path.clone()));

        let runtime = runtime::Builder::new_current_thread().enable_all().build()?;

        runtime.block_on(async move {
            let listener = TcpListener::bind(addr).await?;
            axum::serve(listener, app).await?;
            Ok(())
        })
    }

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

    async fn handle_file(
        State(root): State<Arc<PathBuf>>,
        Path(file_path): Path<String>,
        Query(params): Query<DownloadParams>,
    ) -> Result<Response<Body>, (StatusCode, String)> {
        let path = root.join(file_path.clone());
        // check whether path is a file
        if !path.exists() || !path.is_file() {
            // TODO: Redirect to 404.html
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
            if params.attachment.unwrap_or(false) || Self::force_download(mime_type.as_ref()) {
                format!(
                    "attachment; filename=\"{}\"",
                    Self::safe_filename(&file_path)
                )
            } else {
                format!("inline; filename=\"{}\"", Self::safe_filename(&file_path))
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

    async fn handle_last_modify(
        State(root): State<Arc<PathBuf>>,
    ) -> Result<Json<HashMap<String, u64>>, (StatusCode, String)> {
        match Self::last_modify(root).await {
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
                    Self::safe_filename(path.file_name().map(|s| s.to_str())??),
                    modified_time.duration_since(UNIX_EPOCH).ok()?.as_secs(),
                );
            }
        }

        Some(files)
    }
}
