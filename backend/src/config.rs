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
        let cache_ttl_seconds:u64= env::var("CACHE_TTL_SECONDS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok()) 
            .unwrap_or(DEFAULT_REDIS_TTL as u64) ;

        Ok(Self {
            bind_address,
            database_url,
            redis_url,
            ai_service_url,
            cache_ttl_seconds,
        })
    }
}
