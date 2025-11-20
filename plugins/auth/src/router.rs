//! Authentication plugin for user registration and login
use crate::AuthError;
use crate::conn_pool::ConnPool;
use crate::model::{NewSession, NewUser, Session, User};
use crate::schema::*;
use crate::verify;
use allay_plugin_api::RouteComponent;
use allay_plugin_api::http::response::IntoResponse;
use allay_plugin_api::http::{
    Header, Method, Request, Response, StatusCode, unimplemented_response,
};
use chrono::{Duration, NaiveDateTime, Utc};
use diesel::RunQueryDsl;
use diesel::prelude::*;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::ops::DerefMut;
use std::{fs, path};
use uuid::Uuid;

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

/// Response structure for user profile
#[derive(Debug, Serialize)]
struct ProfileResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
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

        (status, response).into_response()
    }
}

pub type AuthResult<T> = Result<T, AuthError>;

/// Deserializes the request body into the specified type
fn deserialize_body<T: DeserializeOwned>(request: Request) -> AuthResult<T> {
    serde_json::from_slice(request.body()).map_err(|_| AuthError::InvalidPayload)
}

pub struct AuthRouter {
    conn_pool: ConnPool,
}

impl AuthRouter {
    const TOKEN_EXPIRY: Duration = Duration::hours(24);

    pub fn new(db_url: &str) -> Self {
        const EMPTY_DATABASE: &[u8] = include_bytes!("../db/dev.db");

        if !path::Path::new(db_url).exists() {
            fs::write(db_url, EMPTY_DATABASE).expect("Failed to create database file");
        }

        let conn_pool = ConnPool::new(db_url, 4);

        AuthRouter { conn_pool }
    }

    fn create_session(&self, user_id: i32) -> AuthResult<String> {
        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + Self::TOKEN_EXPIRY;

        let session = NewSession {
            token: &token,
            user_id,
            expires_at: expires_at.naive_utc(),
        };

        let mut conn = self.conn_pool.get().lock().expect("Failed to acquire lock");
        diesel::insert_into(sessions::table)
            .values(&session)
            .execute(conn.deref_mut())
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(token)
    }

    fn valid_session(&self, user_token: &str) -> AuthResult<User> {
        use crate::schema::sessions::dsl::*;
        use crate::schema::users::dsl::*;

        let mut conn = self.conn_pool.get().lock().expect("Failed to acquire lock");
        let session = sessions
            .filter(token.eq(user_token))
            .first::<Session>(conn.deref_mut())
            .map_err(|_| AuthError::InvalidToken)?;

        if session.expires_at < Utc::now().naive_utc() {
            return Err(AuthError::InvalidToken);
        }

        users
            .filter(id.eq(session.user_id))
            .first::<User>(conn.deref_mut())
            .map_err(|_| AuthError::InvalidToken)
    }

    fn handle_register(&self, request: RegisterRequest) -> AuthResult<AuthResponse> {
        let user = NewUser {
            username: &request.username,
            email: &request.email,
            password_hash: &verify::hash_password(&request.password)?,
        };

        let mut conn = self.conn_pool.get().lock().expect("Failed to acquire lock");
        let user = diesel::insert_into(users::table)
            .values(&user)
            .returning(User::as_returning())
            .get_result(conn.deref_mut())
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        let token = self.create_session(user.id)?;

        let response = AuthResponse {
            success: true,
            message: "User registered successfully".to_string(),
            token: Some(token),
            user_id: Some(user.id),
        };

        Ok(response)
    }

    fn handle_login(&self, request: LoginRequest) -> AuthResult<AuthResponse> {
        use crate::schema::users::dsl::*;

        let mut conn = self.conn_pool.get().lock().expect("Failed to acquire lock");
        let user = users
            .filter(username.eq(&request.username))
            .first::<User>(conn.deref_mut())
            .map_err(|_| AuthError::InvalidCredentials)?;

        if verify::verify_password(&request.password, &user.password_hash)? {
            let token = self.create_session(user.id)?;
            let response = AuthResponse {
                success: true,
                message: "Login successful".to_string(),
                token: Some(token),
                user_id: Some(user.id),
            };

            Ok(response)
        } else {
            Err(AuthError::InvalidCredentials)
        }
    }

    fn handle_logout(&self, headers: &[Header]) -> AuthResult<AuthResponse> {
        use crate::schema::sessions::dsl::*;

        let user_token = verify::extract_token_from_headers(headers)?;
        let user = self.valid_session(&user_token)?;

        let mut conn = self.conn_pool.get().lock().expect("Failed to acquire lock");
        let deleted =
            diesel::delete(sessions.filter(token.eq(&user_token)).filter(user_id.eq(user.id)))
                .execute(conn.deref_mut())
                .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        if deleted == 0 {
            return Err(AuthError::InvalidToken);
        }

        let response = AuthResponse {
            success: true,
            message: "Logout successful".to_string(),
            token: None,
            user_id: Some(user.id),
        };

        Ok(response)
    }

    fn handle_profile(&self, headers: &[Header]) -> AuthResult<ProfileResponse> {
        let token = verify::extract_token_from_headers(headers);
        let user = self.valid_session(&token?)?;
        let response = ProfileResponse {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at.as_ref().map(NaiveDateTime::to_string),
        };
        Ok(response)
    }

    fn try_handle(&self, request: Request) -> Result<Response, AuthError> {
        let response = match (request.uri(), request.method()) {
            ("/api/auth/register", &Method::Post) => {
                self.handle_register(deserialize_body(request)?).into_response()
            }

            ("/api/auth/login", &Method::Post) => {
                self.handle_login(deserialize_body(request)?).into_response()
            }

            ("/api/auth/logout", &Method::Post) => {
                self.handle_logout(request.headers()).into_response()
            }

            ("/api/auth/profile", &Method::Get) => {
                self.handle_profile(request.headers()).into_response()
            }
            _ => unimplemented_response(),
        };

        Ok(response)
    }
}

impl RouteComponent for AuthRouter {
    fn handle(&self, request: Request) -> Response {
        self.try_handle(request).unwrap_or_else(|e| e.into_response())
    }

    fn route_paths(&self) -> Vec<(Method, String)> {
        vec![
            (Method::Post, "/api/auth/register".to_string()),
            (Method::Post, "/api/auth/login".to_string()),
            (Method::Post, "/api/auth/logout".to_string()),
            (Method::Get, "/api/auth/profile".to_string()),
        ]
    }
}
