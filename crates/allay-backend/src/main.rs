mod config;

use crate::config::BackendConfig;
use axum::extract::Path;
use axum::routing::get;
use axum::{Router, serve};
use std::sync::LazyLock;

static CONFIG: LazyLock<BackendConfig> = LazyLock::new(|| {
    let config_file_exists = std::path::Path::new("config.toml").exists();
    if config_file_exists {
        BackendConfig::from_config_file("config.toml").unwrap_or_default()
    } else {
        BackendConfig::default()
    }
});

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let app = Router::new().route("/static/{*path}", get(get_file));
    let addr = format!("{}:{}", CONFIG.host, CONFIG.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    log::info!("Listening on {}", addr);
    log::info!("Serving static files from {}", CONFIG.static_files_path);

    serve(listener, app).await?;

    Ok(())
}

async fn get_file(Path(path): Path<String>) -> String {
    let path = std::path::Path::new(&CONFIG.static_files_path).join(&path);
    log::trace!("Getting file {:?}", path);
    format!("You requested the file: {}", path.display())
}
