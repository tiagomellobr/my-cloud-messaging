use anyhow::Context;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub api_key: String,
    pub host: String,
    pub port: u16,
    pub cleanup_interval_hours: u64,
    pub notification_ttl_seconds: u32,
    pub vapid_contact: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?,
            api_key: std::env::var("API_KEY").context("API_KEY must be set")?,
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .context("PORT must be a valid number")?,
            cleanup_interval_hours: std::env::var("CLEANUP_INTERVAL_HOURS")
                .unwrap_or_else(|_| "6".to_string())
                .parse()
                .context("CLEANUP_INTERVAL_HOURS must be a valid number")?,
            notification_ttl_seconds: std::env::var("NOTIFICATION_TTL_SECONDS")
                .unwrap_or_else(|_| "86400".to_string())
                .parse()
                .context("NOTIFICATION_TTL_SECONDS must be a valid number")?,
            vapid_contact: std::env::var("VAPID_CONTACT")
                .unwrap_or_else(|_| "mailto:admin@example.com".to_string()),
        })
    }
}
