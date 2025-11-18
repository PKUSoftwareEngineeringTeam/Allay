use crate::component::export::Response;
pub use crate::http::into_response::IntoResponse;
use axum::http::StatusCode;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct ResponseBuilder {
    status_code: Option<StatusCode>,
    body: Option<Vec<u8>>,
    headers: HashMap<String, Vec<u8>>,
}

impl ResponseBuilder {
    pub fn new() -> Self {
        Self {
            status_code: None,
            body: None,
            headers: HashMap::new(),
        }
    }

    pub fn status(mut self, status_code: StatusCode) -> Self {
        self.status_code = Some(status_code);
        self
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    pub fn header(mut self, key: String, value: Vec<u8>) -> Self {
        self.headers.insert(key, value);
        self
    }

    pub fn build(self) -> Option<Response> {
        let response = Response {
            status_code: self.status_code?.as_u16(),
            body: self.body?,
            headers: self.headers.into_iter().collect(),
        };
        Some(response)
    }
}
