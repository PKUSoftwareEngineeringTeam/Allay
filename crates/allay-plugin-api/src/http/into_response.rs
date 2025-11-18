use crate::component::export::Response;
use crate::http::ResponseBuilder;
use axum::http::StatusCode;
use serde::Serialize;

pub trait IntoResponse {
    /// Create a response.
    #[must_use]
    fn into_response(self) -> Response;
}

impl<T: Serialize> IntoResponse for (StatusCode, T) {
    fn into_response(self) -> Response {
        let (status, body) = self;
        let body = serde_json::to_vec(&body).unwrap();
        let content_type = "application/json".to_string().into_bytes();
        let content_length = body.len().to_string().into_bytes();

        ResponseBuilder::new()
            .body(body)
            .status(status)
            .header("content-type".to_string(), content_type)
            .header("content-length".to_string(), content_length)
            .build()
            .unwrap()
    }
}

impl<T: Serialize, E: IntoResponse> IntoResponse for Result<T, E> {
    fn into_response(self) -> Response {
        match self {
            Ok(value) => (StatusCode::OK, value).into_response(),
            Err(error) => error.into_response(),
        }
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}
