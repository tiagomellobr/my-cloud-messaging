use crate::config::Config;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<Config>,
    pub vapid_private_key_pem: Arc<String>,
    pub vapid_public_key_b64: Arc<String>,
}
