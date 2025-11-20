pub use allay_plugin_host::{Header, Method, Request, Response};

pub fn response_internal_error() -> Response {
    Response {
        status_code: 500,
        headers: vec![],
        body: Vec::new(),
    }
}
