use axum::{extract::{Query, State}, http::StatusCode, Json};

use crate::{
    error::AppError,
    models::notification::{
        BroadcastNotificationRequest, BroadcastResult, ListLogsQuery, SendNotificationRequest,
        SendResult,
    },
    repositories::notification_log_repo,
    services::notification_service,
    state::AppState,
};

pub async fn send(
    State(state): State<AppState>,
    Json(req): Json<SendNotificationRequest>,
) -> Result<(StatusCode, Json<SendResult>), AppError> {
    if req.subscription_ids.is_empty() {
        return Err(AppError::Validation("subscription_ids must not be empty".into()));
    }
    if req.notification.title.is_empty() {
        return Err(AppError::Validation("notification.title is required".into()));
    }

    let ttl = req.ttl.unwrap_or(state.config.notification_ttl_seconds);
    let payload = serde_json::to_value(&req.notification)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    let result = notification_service::send_to_ids(&state, &req.subscription_ids, &payload, ttl)
        .await
        .map_err(|e| AppError::Internal(e))?;

    Ok((StatusCode::OK, Json(result)))
}

pub async fn broadcast(
    State(state): State<AppState>,
    Json(req): Json<BroadcastNotificationRequest>,
) -> Result<(StatusCode, Json<BroadcastResult>), AppError> {
    if req.notification.title.is_empty() {
        return Err(AppError::Validation("notification.title is required".into()));
    }

    let ttl = req.ttl.unwrap_or(state.config.notification_ttl_seconds);
    let payload = serde_json::to_value(&req.notification)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    let result = notification_service::broadcast(&state, &req.tags, &payload, ttl)
        .await
        .map_err(|e| AppError::Internal(e))?;

    Ok((StatusCode::OK, Json(result)))
}

pub async fn log(
    State(state): State<AppState>,
    Query(query): Query<ListLogsQuery>,
) -> Result<Json<Vec<crate::models::notification::NotificationLog>>, AppError> {
    let logs = notification_log_repo::list(&state.db, &query).await?;
    Ok(Json(logs))
}
