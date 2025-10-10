use std::env;

pub const DEFAULT_BIND_ADDRESS: &str = "0.0.0.0:8080";
pub const DEFAULT_AI_URL: &str = "http://localhost:8002";
pub const DEFAULT_REDIS_TTL: usize = 900; // seconds

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub bind_address: String,
    pub database_url: String,
    pub redis_url: String,
    pub ai_service_url: String,
    pub cache_ttl_seconds: u64,
    pub jwt_secret: String,
    pub jwt_ttl_seconds: i64,
    pub admin_username: String,
    pub admin_password: String,
    pub admin_role: String,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let database_url =
            env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL not set"))?;
        let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
        let bind_address =
            env::var("BIND_ADDRESS").unwrap_or_else(|_| DEFAULT_BIND_ADDRESS.to_string());
        let ai_service_url =
            env::var("AI_SERVICE_URL").unwrap_or_else(|_| DEFAULT_AI_URL.to_string());
        let cache_ttl_seconds: u64 = env::var("CACHE_TTL_SECONDS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(DEFAULT_REDIS_TTL as u64);
        let jwt_secret =
            env::var("JWT_SECRET").unwrap_or_else(|_| "development-secret-change-me".into());
        let jwt_ttl_seconds = env::var("JWT_TTL_SECONDS")
            .ok()
            .and_then(|value| value.parse::<i64>().ok())
            .unwrap_or(3600);
        let admin_username = env::var("ADMIN_USERNAME").unwrap_or_else(|_| "admin".into());
        let admin_password = env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin123456".into());
        let admin_role = env::var("ADMIN_ROLE").unwrap_or_else(|_| "admin".into());

        Ok(Self {
            bind_address,
            database_url,
            redis_url,
            ai_service_url,
            cache_ttl_seconds,
            jwt_secret,
            jwt_ttl_seconds,
            admin_username,
            admin_password,
            admin_role,
        })
    }
}
