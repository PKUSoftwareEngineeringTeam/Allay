use crate::Event;
use axum::Router;
use std::{path::PathBuf, sync::RwLock};

#[derive(Default)]
pub struct RouteRegisterEvent {
    path: PathBuf,
    app: RwLock<Option<Router>>,
}

impl Event for RouteRegisterEvent {}

impl RouteRegisterEvent {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            app: RwLock::new(Some(Router::new())),
        }
    }

    pub fn route(&self, callback: impl FnOnce(Router) -> Router) {
        let mut app_lock = self.app.write().unwrap();
        let app = app_lock.take().unwrap();
        *app_lock = Some(callback(app));
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn take_app(&self) -> Router<()> {
        let mut app_lock = self.app.write().unwrap();
        app_lock.take().unwrap()
    }
}
