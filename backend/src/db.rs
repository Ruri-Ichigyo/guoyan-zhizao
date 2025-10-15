use chrono::Utc;
use sqlx::{postgres::PgPoolOptions, PgPool};

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

const CREATE_DETECTIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS detections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    snapshot BYTEA NOT NULL,
    labels TEXT[] NOT NULL,
    confidence_scores REAL[] NOT NULL,
    detected_classes TEXT[] NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    source TEXT DEFAULT 'live_camera'
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
    sqlx::query(CREATE_DETECTIONS_TABLE).execute(&pool).await?;

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
