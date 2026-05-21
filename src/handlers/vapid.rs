use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::state::AppState;

pub async fn get_public_key(State(state): State<AppState>) -> Json<Value> {
    Json(json!({ "public_key": *state.vapid_public_key_b64 }))
}
