//! A simple HTTP server.
use crate::ServerResult;
use crate::builtin::BuiltinRoutePlugin;
use crate::routes::RouteEvent;
use allay_plugin::PluginManager;
use axum::Router;
use std::path::{self, PathBuf};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::runtime;
use tracing::warn;

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
        let runtime = runtime::Builder::new_current_thread().enable_all().build()?;
        runtime.block_on(async move {
            let addr = format!("{}:{}", self.host, self.port);
            let app = self.router();
            let listener = TcpListener::bind(addr).await?;
            axum::serve(listener, app).await?;
            Ok(())
        })
    }

    /// Builds the Axum router for the server.
    fn router(&self) -> Router {
        let manager = PluginManager::instance();
        if let Err(e) = manager.register_plugin(Arc::new(BuiltinRoutePlugin)) {
            warn!("Failed to register BuiltinRoutePlugin: {}", e);
        };

        let mut event = RouteEvent::new();
        manager.event_bus().publish(&mut event);
        event.app().with_state(Arc::new(self.path.clone()))
    }
}
