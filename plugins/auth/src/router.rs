//! Authentication plugin for user registration and login
use crate::AuthError;
use crate::model::NewUser;
use crate::model::User;
use crate::schema::*;
use crate::verify;
use allay_plugin_api::route::TryRouteComponent;
use allay_plugin_api::route::unimplemented_response;
use axum::Json;
use axum::body::to_bytes;
use axum::extract::Request;
use axum::http::Method;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use diesel::RunQueryDsl;
use diesel::prelude::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path;

/// User data for registration
#[derive(Debug, Deserialize)]
struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// User data for login
#[derive(Debug, Deserialize)]
struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Response structure for authentication operations
#[derive(Debug, Serialize)]
struct AuthResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i32>,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AuthError::UserExists => (StatusCode::CONFLICT, "User already exists".to_string()),
            AuthError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string())
            }
            AuthError::InvalidPayload => (StatusCode::BAD_REQUEST, "Invalid payload".to_string()),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token".to_string()),
            AuthError::HashingError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Password hashing error".to_string(),
            ),
        };

        let response = AuthResponse {
            success: false,
            message,
            token: None,
            user_id: None,
        };

        (status, Json(response)).into_response()
    }
}

pub type AuthResult<T> = std::result::Result<T, AuthError>;

/// Deserializes the request body into the specified type
async fn deserialize_body<T: DeserializeOwned>(request: Request) -> AuthResult<T> {
    let bytes = to_bytes(request.into_body(), usize::MAX).await.unwrap_or_default();
    serde_json::from_slice(&bytes).map_err(|_| AuthError::InvalidPayload)
}

pub struct AuthRouter {
    db_url: String,
}

impl AuthRouter {
    pub fn new(database_url: &str) -> Self {
        AuthRouter {
            db_url: database_url.to_string(),
        }
    }
}

impl AuthRouter {
    fn create_conn(&self) -> SqliteConnection {
        const EMPTY_DATABASE: &[u8] = include_bytes!("../db/dev.db");

        if !path::Path::new(&self.db_url).exists() {
            fs::write(&self.db_url, EMPTY_DATABASE).expect("Failed to create database file");
        }

        SqliteConnection::establish(&self.db_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", &self.db_url))
    }

    fn handle_register(&self, request: RegisterRequest) -> AuthResult<Json<AuthResponse>> {
        let user = NewUser {
            username: &request.username,
            email: &request.email,
            password_hash: &verify::hash_password(&request.password)?,
        };

        let user = diesel::insert_into(user::table)
            .values(&user)
            .returning(User::as_returning())
            .get_result(&mut self.create_conn())
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        let response = AuthResponse {
            success: true,
            message: "User registered successfully".to_string(),
            token: None,
            user_id: Some(user.id),
        };

        Ok(Json(response))
    }
}

#[async_trait::async_trait]
impl TryRouteComponent for AuthRouter {
    type Error = AuthError;

    async fn try_handle(&self, request: Request) -> Result<Response, AuthError> {
        // match request path and method
        let response = match (request.uri().path(), request.method()) {
            ("/api/auth/register", &Method::POST) => {
                self.handle_register(deserialize_body(request).await?).into_response()
            }

            // ("/api/auth/login", &Method::POST) => {
            //     self.handle_login(deserialize_body(request).await?).await.into_response()
            // }

            // ("/api/auth/logout", &Method::POST) => {
            //     self.handle_logout(request.headers().clone()).await.into_response()
            // }

            // ("/api/auth/profile", &Method::GET) => {
            //     self.handle_profile(request.headers().clone()).await.into_response()
            // }
            _ => unimplemented_response(),
        };

        Ok(response)
    }
}
