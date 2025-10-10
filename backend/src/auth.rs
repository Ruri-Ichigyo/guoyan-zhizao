use std::ops::Add;

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{header, request::Parts},
};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use tracing::warn;
use uuid::Uuid;

use crate::{errors::AppError, models::UserProfile, AppState};

pub const ROLE_ADMIN: &str = "admin";
pub const ROLE_OPERATOR: &str = "operator";
pub const ROLE_VIEWER: &str = "viewer";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub user_id: Uuid,
    pub exp: usize,
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub username: String,
    pub role: String,
}

impl AuthUser {
    pub fn to_profile(&self) -> UserProfile {
        UserProfile {
            user_id: self.user_id,
            username: self.username.clone(),
            role: self.role.clone(),
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .ok_or_else(|| AppError::unauthorized("缺少认证信息"))?;

        let auth_value = auth_header
            .to_str()
            .map_err(|_| AppError::unauthorized("认证信息格式无效"))?;

        let token = auth_value
            .strip_prefix("Bearer ")
            .or_else(|| auth_value.strip_prefix("bearer "))
            .ok_or_else(|| AppError::unauthorized("认证信息格式无效"))?;

        let claims = validate_token(token, app_state.jwt_secret.as_ref())?;

        Ok(AuthUser {
            user_id: claims.user_id,
            username: claims.sub,
            role: claims.role,
        })
    }
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|err| AppError::Unexpected(err.to_string()))
        .map(|hash| hash.to_string())
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    match PasswordHash::new(password_hash) {
        Ok(parsed) => Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()),
        Err(err) => {
            warn!("Invalid password hash encountered: {err}");
            Ok(false)
        }
    }
}

pub fn generate_token(
    user_id: Uuid,
    username: &str,
    role: &str,
    secret: &str,
    ttl: Duration,
) -> Result<(String, DateTime<Utc>), AppError> {
    let expires_at = Utc::now().add(ttl);
    let claims = Claims {
        sub: username.to_string(),
        role: role.to_string(),
        user_id,
        exp: expires_at.timestamp() as usize,
    };

    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|err| AppError::Unexpected(err.to_string()))?;

    Ok((token, expires_at))
}

pub fn validate_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    let validation = Validation::default();
    let data = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|_| AppError::unauthorized("凭证无效或已过期"))?;

    Ok(data.claims)
}

pub fn ensure_role(user: &AuthUser, allowed_roles: &[&str]) -> Result<(), AppError> {
    if allowed_roles
        .iter()
        .any(|role| user.role.eq_ignore_ascii_case(role))
    {
        Ok(())
    } else {
        Err(AppError::forbidden("缺少访问权限"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_roundtrip() {
        let password = "S3cret!";
        let hash = hash_password(password).expect("hash");
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong", &hash).unwrap());
    }

    #[test]
    fn token_cycle() {
        let user_id = Uuid::new_v4();
        let (token, expires_at) = generate_token(
            user_id,
            "tester",
            ROLE_ADMIN,
            "secret",
            Duration::minutes(15),
        )
        .unwrap();
        assert!(expires_at > Utc::now());
        let claims = validate_token(&token, "secret").unwrap();
        assert_eq!(claims.user_id, user_id);
        assert_eq!(claims.role, ROLE_ADMIN);
        assert_eq!(claims.sub, "tester");
    }

    #[test]
    fn role_enforcement() {
        let user = AuthUser {
            user_id: Uuid::new_v4(),
            username: "demo".into(),
            role: ROLE_OPERATOR.into(),
        };

        assert!(ensure_role(&user, &[ROLE_OPERATOR, ROLE_ADMIN]).is_ok());
        assert!(ensure_role(&user, &[ROLE_ADMIN]).is_err());
    }
}
