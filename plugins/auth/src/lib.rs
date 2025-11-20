use crate::router::AuthRouter;
use allay_plugin_api::{Plugin, RouteComponent, register_plugin};
mod conn_pool;
mod model;
mod router;
mod schema;
mod verify;

extern crate libsqlite3_sys;

/// Authentication error types
#[derive(Debug)]
pub enum AuthError {
    DatabaseError(String),
    UserExists,
    InvalidPayload,
    InvalidCredentials,
    InvalidToken,
    HashingError,
}

pub struct AuthPlugin {
    router: AuthRouter,
}

impl Plugin for AuthPlugin {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "authentication"
    }

    fn version() -> &'static str
    where
        Self: Sized,
    {
        "0.1.0"
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        let db_url = "auth.db";
        AuthPlugin {
            router: AuthRouter::new(db_url),
        }
    }

    fn route_component(&self) -> &dyn RouteComponent {
        &self.router
    }
}

register_plugin!(AuthPlugin);
