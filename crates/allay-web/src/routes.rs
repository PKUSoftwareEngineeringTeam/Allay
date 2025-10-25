use allay_plugin::Event;
use axum::Router;
use axum::routing::MethodRouter;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Default)]
pub struct RouteEvent {
    app: Option<Router<Arc<PathBuf>>>,
}

impl Event for RouteEvent {}

impl RouteEvent {
    pub(crate) fn new() -> Self {
        Self {
            app: Some(Router::new()),
        }
    }

    pub fn register(&mut self, path: &str, router: MethodRouter<Arc<PathBuf>>) {
        self.app = Some(self.app.take().unwrap().route(path, router));
    }

    pub(crate) fn app(self) -> Router<Arc<PathBuf>> {
        self.app.unwrap()
    }
}
