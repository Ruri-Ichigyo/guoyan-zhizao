use chrono::Utc;
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

use crate::{auth, errors::AppError, models::UserRecord};

const CREATE_UPLOADS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS uploads (
    id UUID PRIMARY KEY,
    filename TEXT NOT NULL,
    content_type TEXT,
    data BYTEA NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
"#;

const CREATE_RESULTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS results (
    upload_id UUID PRIMARY KEY REFERENCES uploads(id) ON DELETE CASCADE,
    status TEXT NOT NULL,
    detections JSONB NOT NULL,
    total_fruits INTEGER NOT NULL DEFAULT 0,
    grade_a INTEGER NOT NULL DEFAULT 0,
    grade_b INTEGER NOT NULL DEFAULT 0,
    grade_c INTEGER NOT NULL DEFAULT 0,
    avg_confidence REAL NOT NULL DEFAULT 0.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
"#;

const CREATE_USERS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
"#;

pub async fn init_pool(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Ensure tables exist
    sqlx::query(CREATE_UPLOADS_TABLE).execute(&pool).await?;
    sqlx::query(CREATE_RESULTS_TABLE).execute(&pool).await?;
    sqlx::query(CREATE_USERS_TABLE).execute(&pool).await?;

    Ok(pool)
}

pub async fn mark_upload_failed(pool: &PgPool, upload_id: uuid::Uuid) -> anyhow::Result<()> {
    sqlx::query("UPDATE uploads SET status = $1, updated_at = $2 WHERE id = $3")
        .bind("failed")
        .bind(Utc::now())
        .bind(upload_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn ensure_default_user(
    pool: &PgPool,
    username: &str,
    password: &str,
    role: &str,
) -> Result<(), AppError> {
    let password_hash = auth::hash_password(password)?;
    sqlx::query(
        "INSERT INTO users (id, username, password_hash, role) VALUES ($1, $2, $3, $4)
         ON CONFLICT (username) DO NOTHING",
    )
    .bind(Uuid::new_v4())
    .bind(username)
    .bind(password_hash)
    .bind(role)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn find_user_by_username(
    pool: &PgPool,
    username: &str,
) -> Result<Option<UserRecord>, sqlx::Error> {
    sqlx::query_as::<_, UserRecord>(
        "SELECT id, username, password_hash, role FROM users WHERE username = $1",
    )
    .bind(username)
    .fetch_optional(pool)
    .await
}
