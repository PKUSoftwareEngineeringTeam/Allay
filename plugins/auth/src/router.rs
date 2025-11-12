//! Authentication plugin for user registration and login
use allay_plugin_api::route::TryRouteComponent;
use allay_plugin_api::route::unimplemented_response;
use axum::Json;
use axum::body::to_bytes;
use axum::extract::Request;
use axum::http::Method;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, sqlite::SqlitePoolOptions};
use std::sync::Arc;
use tokio::sync::OnceCell;
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
    pub user_id: Option<String>,
}

/// User information stored in database
#[derive(Debug, Serialize)]
struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: String,
}

/// Database connection state
#[derive(Clone)]
struct DbState {
    pub pool: Pool<Sqlite>,
}

/// Session manager for token validation
#[derive(Clone)]
struct SessionManager {
    pub db_state: DbState,
}

impl SessionManager {
    /// Creates a new session manager instance
    pub fn new(db_state: DbState) -> Self {
        Self { db_state }
    }

    /// Validates a user session token
    pub async fn validate_session(&self, token: &str) -> Result<User, AuthError> {
        let result = sqlx::query(
            "SELECT u.id, u.username, u.email, u.created_at 
             FROM users u 
             INNER JOIN sessions s ON u.id = s.user_id 
             WHERE s.token = ? AND s.expires_at > datetime('now')",
        )
        .bind(token)
        .fetch_optional(&self.db_state.pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        match result {
            Some(row) => {
                let user = User {
                    id: row.get("id"),
                    username: row.get("username"),
                    email: row.get("email"),
                    created_at: row.get("created_at"),
                };
                Ok(user)
            }
            None => Err(AuthError::InvalidToken),
        }
    }

    /// Creates a new session for a user
    pub async fn create_session(&self, user_id: &str) -> Result<String, AuthError> {
        let token = Uuid::new_v4().to_string();
        let expires_at = chrono::Utc::now() + chrono::Duration::days(7);

        sqlx::query("INSERT INTO sessions (token, user_id, expires_at) VALUES (?, ?, ?)")
            .bind(&token)
            .bind(user_id)
            .bind(expires_at.to_rfc3339())
            .execute(&self.db_state.pool)
            .await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        Ok(token)
    }
}

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

/// Hashes a password using bcrypt
fn hash_password(password: &str) -> Result<String, AuthError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(|_| AuthError::HashingError)
}

/// Verifies a password against a hash
fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    bcrypt::verify(password, hash).map_err(|_| AuthError::HashingError)
}

/// Handles user registration
async fn handle_register(
    db_state: Arc<DbState>,
    payload: RegisterRequest,
) -> Result<Json<AuthResponse>, AuthError> {
    // Check if user already exists
    let existing_user = sqlx::query("SELECT id FROM users WHERE username = ? OR email = ?")
        .bind(&payload.username)
        .bind(&payload.email)
        .fetch_optional(&db_state.pool)
        .await
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    if existing_user.is_some() {
        return Err(AuthError::UserExists);
    }

    // Hash password
    let password_hash = hash_password(&payload.password)?;

    // Create user
    let user_id = Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&user_id)
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&created_at)
    .execute(&db_state.pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    let response = AuthResponse {
        success: true,
        message: "User registered successfully".to_string(),
        token: None,
        user_id: Some(user_id),
    };

    Ok(Json(response))
}

/// Handles user login
async fn handle_login(
    state: Arc<AuthState>,
    payload: LoginRequest,
) -> Result<Json<AuthResponse>, AuthError> {
    // Find user by username
    let user_result = sqlx::query(
        "SELECT id, username, email, password_hash, created_at FROM users WHERE username = ?",
    )
    .bind(&payload.username)
    .fetch_optional(&state.db_state.pool)
    .await
    .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    let user_row = match user_result {
        Some(row) => row,
        None => return Err(AuthError::InvalidCredentials),
    };

    // Verify password
    let password_hash: String = user_row.get("password_hash");
    if !verify_password(&payload.password, &password_hash)? {
        return Err(AuthError::InvalidCredentials);
    }

    // Create session
    let user_id: String = user_row.get("id");
    let token = state.session_manager.create_session(&user_id).await?;

    let response = AuthResponse {
        success: true,
        message: "Login successful".to_string(),
        token: Some(token),
        user_id: Some(user_id),
    };

    Ok(Json(response))
}

/// Handles user logout
async fn handle_logout(
    state: Arc<AuthState>,
    headers: HeaderMap,
) -> Result<Json<AuthResponse>, AuthError> {
    if let Some(auth_header) = headers.get("authorization")
        && let Ok(token) = auth_header.to_str()
        && token.starts_with("Bearer ")
    {
        let token = token.trim_start_matches("Bearer ");
        sqlx::query("DELETE FROM sessions WHERE token = ?")
            .bind(token)
            .execute(&state.db_state.pool)
            .await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;
    }

    let response = AuthResponse {
        success: true,
        message: "Logout successful".to_string(),
        token: None,
        user_id: None,
    };

    Ok(Json(response))
}

/// Handles getting current user profile
async fn handle_profile(
    state: Arc<AuthState>,
    headers: HeaderMap,
) -> Result<Json<User>, AuthError> {
    let token = extract_token_from_headers(headers)?;
    let user = state.session_manager.validate_session(&token).await?;
    Ok(Json(user))
}

/// Extracts token from authorization headers
fn extract_token_from_headers(headers: HeaderMap) -> Result<String, AuthError> {
    let auth_header = headers
        .get("authorization")
        .ok_or(AuthError::InvalidToken)?
        .to_str()
        .map_err(|_| AuthError::InvalidToken)?;

    if auth_header.starts_with("Bearer ") {
        Ok(auth_header.trim_start_matches("Bearer ").to_string())
    } else {
        Err(AuthError::InvalidToken)
    }
}

/// Authentication state shared across routes
#[derive(Clone)]
struct AuthState {
    pub db_state: Arc<DbState>,
    pub session_manager: SessionManager,
}

impl AuthState {
    /// Creates new authentication state
    pub fn new(db_state: DbState) -> Self {
        let session_manager = SessionManager::new(db_state.clone());
        Self {
            db_state: Arc::new(db_state),
            session_manager,
        }
    }
}

/// Initializes database tables
async fn init_database(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    // Create users table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await?;

    // Create sessions table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sessions (
            token TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            created_at TEXT DEFAULT (datetime('now')),
            FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
        )",
    )
    .execute(pool)
    .await?;

    // Create indexes for better performance
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id)")
        .execute(pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at)")
        .execute(pool)
        .await?;

    Ok(())
}

pub struct AuthRouter {
    database_url: String,
    auth_state: OnceCell<Arc<AuthState>>,
}

impl AuthRouter {
    pub fn new(database_url: &str) -> Self {
        AuthRouter {
            database_url: database_url.to_string(),
            auth_state: OnceCell::new(),
        }
    }
}

async fn deserialize_body<T: DeserializeOwned>(request: Request) -> Result<T, AuthError> {
    let bytes = to_bytes(request.into_body(), usize::MAX).await.unwrap_or_default();
    serde_json::from_slice(&bytes).map_err(|_| AuthError::InvalidPayload)
}

#[async_trait::async_trait]
impl TryRouteComponent for AuthRouter {
    type Error = AuthError;

    async fn try_handle(&self, request: Request) -> Result<Response, AuthError> {
        // Initialize auth state if not already done
        let auth_state = self
            .auth_state
            .get_or_init(async || {
                let pool = SqlitePoolOptions::new()
                    .connect(&self.database_url)
                    .await
                    .expect("Failed to connect to database");
                init_database(&pool).await.expect("Failed to initialize database");
                let db_state = DbState { pool };
                Arc::new(AuthState::new(db_state))
            })
            .await;

        // match request path and method
        let response = match (request.uri().path(), request.method()) {
            ("/api/auth/register", &Method::POST) => handle_register(
                auth_state.db_state.clone(),
                deserialize_body(request).await?,
            )
            .await
            .into_response(),

            ("/api/auth/login", &Method::POST) => {
                handle_login(auth_state.clone(), deserialize_body(request).await?)
                    .await
                    .into_response()
            }

            ("/api/auth/logout", &Method::POST) => {
                handle_logout(auth_state.clone(), request.headers().clone())
                    .await
                    .into_response()
            }

            ("/api/auth/profile", &Method::GET) => {
                handle_profile(auth_state.clone(), request.headers().clone())
                    .await
                    .into_response()
            }

            _ => unimplemented_response(),
        };

        Ok(response)
    }
}
