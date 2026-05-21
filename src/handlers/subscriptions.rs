use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{
    error::AppError,
    models::subscription::{
        CreateSubscriptionRequest, ListSubscriptionsQuery, SubscriptionCreated, SubscriptionResponse,
    },
    repositories::subscription_repo,
    state::AppState,
};

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<CreateSubscriptionRequest>,
) -> Result<(StatusCode, Json<SubscriptionCreated>), AppError> {
    if req.endpoint.is_empty() {
        return Err(AppError::Validation("endpoint is required".into()));
    }
    if req.keys.p256dh.is_empty() || req.keys.auth.is_empty() {
        return Err(AppError::Validation("keys.p256dh and keys.auth are required".into()));
    }

    if let Some(existing) = subscription_repo::find_by_endpoint(&state.db, &req.endpoint).await? {
        return Err(AppError::SubscriptionExists(existing.id));
    }

    let subscription = subscription_repo::create(
        &state.db,
        &req.endpoint,
        &req.keys.p256dh,
        &req.keys.auth,
        &req.tags,
        req.metadata,
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(SubscriptionCreated {
            id: subscription.id,
            created_at: subscription.created_at,
        }),
    ))
}

pub async fn remove(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let deleted = subscription_repo::delete(&state.db, id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<ListSubscriptionsQuery>,
) -> Result<Json<Vec<SubscriptionResponse>>, AppError> {
    let subs = subscription_repo::list(&state.db, &query).await?;
    Ok(Json(subs.into_iter().map(SubscriptionResponse::from).collect()))
}
