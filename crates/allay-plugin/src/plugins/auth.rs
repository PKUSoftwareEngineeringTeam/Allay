//! Authentication plugin for user registration and login
use crate::bus::AsyncEventHandler;
use crate::events::RouteRegisterEvent;
use crate::{Plugin, PluginContext};
use async_trait::async_trait;
use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, sqlite::SqlitePoolOptions};
use std::sync::Arc;
use tokio::sync::RwLock;
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
enum AuthError {
    DatabaseError(String),
    UserExists,
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
    State(db_state): State<Arc<DbState>>,
    Json(payload): Json<RegisterRequest>,
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
    State(state): State<Arc<AuthState>>,
    Json(payload): Json<LoginRequest>,
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
    State(state): State<Arc<AuthState>>,
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
    State(state): State<Arc<AuthState>>,
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

/// Authentication route handler
pub struct AuthRouteHandler {
    database_url: String,
    auth_state: RwLock<Option<Arc<AuthState>>>,
}

impl AuthRouteHandler {
    /// Creates a new authentication route handler
    fn new(database_url: String) -> Self {
        Self {
            database_url,
            auth_state: RwLock::new(None),
        }
    }
}

#[async_trait]
impl AsyncEventHandler<RouteRegisterEvent> for AuthRouteHandler {
    async fn handle_event(self: Arc<Self>, event: Arc<RouteRegisterEvent>) -> anyhow::Result<()> {
        // init the pool if unnot done yet
        let mut lock = self.auth_state.write().await;
        let auth_state = if let Some(state) = lock.as_ref() {
            state.clone()
        } else {
            let pool = SqlitePoolOptions::new().connect(&self.database_url).await?;
            init_database(&pool).await?;
            let db_state = DbState { pool };
            let auth_state = Arc::new(AuthState::new(db_state));
            // store it for future use
            *lock = Some(auth_state.clone());
            auth_state
        };

        let db_state = auth_state.db_state.clone();
        event.route(|app| {
            app.with_state(())
                .route("/api/auth/register", post(handle_register))
                .with_state(db_state)
                .route("/api/auth/login", post(handle_login))
                .route("/api/auth/logout", post(handle_logout))
                .route("/api/auth/profile", get(handle_profile))
                .with_state(auth_state)
        });
        Ok(())
    }
}

pub struct AuthPlugin;

impl Plugin for AuthPlugin {
    fn name(&self) -> &str {
        "auth-plugin"
    }

    fn initialize(&self, context: PluginContext) -> anyhow::Result<()> {
        let database_url = match self.config().get("database") {
            Some(url) => url.as_str()?,
            None => "sqlite:auth.db",
        }
        .to_string();


        let handler = Arc::new(AuthRouteHandler::new(database_url));
        context.event_bus.register_async_handler(handler);

        Ok(())
    }
}
