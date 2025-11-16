use crate::AuthError;
use axum::http::HeaderMap;

/// Hashes a password using bcrypt
pub fn hash_password(password: &str) -> Result<String, AuthError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(|_| AuthError::HashingError)
}

/// Verifies a password against a hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    bcrypt::verify(password, hash).map_err(|_| AuthError::HashingError)
}

/// Extracts token from authorization headers
pub fn extract_token_from_headers(headers: HeaderMap) -> Result<String, AuthError> {
    let auth_header = headers
        .get("authorization")
        .ok_or(AuthError::InvalidToken)?
        .to_str()
        .map_err(|_| AuthError::InvalidToken)?;

    if auth_header.len() >= 7 && auth_header[..7].eq_ignore_ascii_case("Bearer ") {
        Ok(auth_header[7..].to_string())
    } else {
        Err(AuthError::InvalidToken)
    }
}
