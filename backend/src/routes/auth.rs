use axum::{extract::State, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    auth::{generate_token, verify_password, AuthUser},
    db::find_user_by_username,
    errors::AppError,
    models::UserProfile,
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub user: UserProfile,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<LoginResponse>, AppError> {
    let Some(record) = find_user_by_username(&state.pool, &payload.username).await? else {
        return Err(AppError::unauthorized("用户名或密码错误"));
    };

    if !verify_password(&payload.password, &record.password_hash)? {
        return Err(AppError::unauthorized("用户名或密码错误"));
    }

    let (token, expires_at) = generate_token(
        record.id,
        &record.username,
        &record.role,
        state.jwt_secret.as_ref(),
        state.jwt_ttl,
    )?;

    let profile = UserProfile {
        user_id: record.id,
        username: record.username,
        role: record.role,
    };

    Ok(Json(LoginResponse {
        token,
        expires_at,
        user: profile,
    }))
}

pub async fn me(user: AuthUser) -> Result<Json<UserProfile>, AppError> {
    Ok(Json(user.to_profile()))
}
