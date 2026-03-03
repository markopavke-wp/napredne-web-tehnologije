use axum::{extract::State, Json};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::user::{AuthResponse, LoginRequest, RegisterRequest, User, UserResponse};
use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    if body.username.len() < 3 {
        return Err(AppError::BadRequest("Username must be at least 3 characters".into()));
    }
    if body.password.len() < 6 {
        return Err(AppError::BadRequest("Password must be at least 6 characters".into()));
    }

    let password_hash = hash(&body.password, DEFAULT_COST)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, username, email, password_hash, created_at)
        VALUES ($1, $2, $3, $4, NOW())
        RETURNING id, username, email, password_hash, created_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(&body.username)
    .bind(&body.email)
    .bind(&password_hash)
    .fetch_one(&state.db)
    .await
    .map_err(|_| AppError::Conflict("Email or username already exists".into()))?;

    let token = generate_token(&user.id.to_string(), &state.jwt_secret)?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, created_at FROM users WHERE email = $1",
    )
    .bind(&body.email)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::Unauthorized)?;

    let valid = verify(&body.password, &user.password_hash)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if !valid {
        return Err(AppError::Unauthorized);
    }

    let token = generate_token(&user.id.to_string(), &state.jwt_secret)?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

fn generate_token(user_id: &str, secret: &str) -> Result<String, AppError> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .unwrap()
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|e| AppError::Internal(e.to_string()))
}