mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod models;
mod repositories;
mod services;
mod state;

use axum::{
    middleware as axum_middleware,
    routing::{delete, get, post},
    Router,
};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::{
    config::Config,
    middleware::auth::require_bearer,
    services::vapid_service,
    state::AppState,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,sqlx=warn".parse().unwrap()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;
    tracing::info!("Starting my-cloud-messaging v{}", env!("CARGO_PKG_VERSION"));

    let pool = db::create_pool(&config.database_url).await?;
    tracing::info!("Database connected");

    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Migrations applied");

    let vapid_keys = vapid_service::init(&pool).await?;
    tracing::info!(public_key = %vapid_keys.public_key_b64, "VAPID ready");

    let state = AppState {
        db: pool.clone(),
        config: Arc::new(config.clone()),
        vapid_private_key_pem: Arc::new(vapid_keys.private_key_pem),
        vapid_public_key_b64: Arc::new(vapid_keys.public_key_b64),
    };

    spawn_cleanup_task(state.clone(), config.cleanup_interval_hours);

    let app = build_router(state);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn build_router(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/health", get(handlers::health::handler))
        .route("/vapid-public-key", get(handlers::vapid::get_public_key))
        .route("/subscriptions", post(handlers::subscriptions::register));

    let protected_routes = Router::new()
        .route("/subscriptions", get(handlers::subscriptions::list))
        .route("/subscriptions/{id}", delete(handlers::subscriptions::remove))
        .route("/notifications/send", post(handlers::notifications::send))
        .route("/notifications/broadcast", post(handlers::notifications::broadcast))
        .route("/notifications/log", get(handlers::notifications::log))
        .layer(axum_middleware::from_fn_with_state(state.clone(), require_bearer));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::with_status_code(axum::http::StatusCode::REQUEST_TIMEOUT, Duration::from_secs(30)))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

fn spawn_cleanup_task(state: AppState, interval_hours: u64) {
    tokio::spawn(async move {
        let interval = Duration::from_secs(interval_hours * 3600);
        loop {
            tokio::time::sleep(interval).await;
            match repositories::subscription_repo::cleanup_gone(&state.db).await {
                Ok(n) if n > 0 => tracing::info!("Cleaned up {} gone subscriptions", n),
                Ok(_) => {}
                Err(e) => tracing::warn!("Cleanup failed: {}", e),
            }
        }
    });
}
