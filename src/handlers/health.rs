use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};

use crate::state::AppState;

pub async fn handler(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    let db_ok = sqlx::query("SELECT 1").execute(&state.db).await.is_ok();

    if db_ok {
        (
            StatusCode::OK,
            Json(json!({
                "status": "ok",
                "database": "ok",
                "version": env!("CARGO_PKG_VERSION")
            })),
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "status": "degraded",
                "database": "error",
                "version": env!("CARGO_PKG_VERSION")
            })),
        )
    }
}
