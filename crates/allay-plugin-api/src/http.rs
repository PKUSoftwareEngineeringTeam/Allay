mod into_response;
pub mod request;
pub mod response;

pub use crate::component::export::{Header, Method, Request, Response};
use crate::http::response::ResponseBuilder;
pub use axum::http::StatusCode;

/// Default response for unimplemented routes
/// Returns a 501 Not Implemented response
pub fn unimplemented_response() -> Response {
    ResponseBuilder::new()
        .status(StatusCode::NOT_IMPLEMENTED)
        .body(Vec::new())
        .build()
        .unwrap()
}
