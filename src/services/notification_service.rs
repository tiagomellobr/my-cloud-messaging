use crate::{
    models::notification::{BroadcastResult, SendResult, SendResultItem},
    repositories::{notification_log_repo, subscription_repo},
    services::vapid_service::{self, PushResult},
    state::AppState,
};
use futures::stream::{self, StreamExt};
use serde_json::Value;
use uuid::Uuid;

pub async fn send_to_ids(
    state: &AppState,
    subscription_ids: &[Uuid],
    payload: &Value,
    ttl: u32,
) -> anyhow::Result<SendResult> {
    let subscriptions = subscription_repo::find_by_ids(&state.db, subscription_ids).await?;
    let total = subscriptions.len();

    let results: Vec<SendResultItem> = stream::iter(subscriptions)
        .map(|sub| {
            let state = state.clone();
            let payload = payload.clone();
            async move {
                let result = vapid_service::send_push(
                    &sub.endpoint,
                    &sub.p256dh,
                    &sub.auth,
                    &payload,
                    &state.vapid_private_key_pem,
                    &state.config.vapid_contact,
                    ttl,
                )
                .await;

                log_result(&state, sub.id, &sub.endpoint, &payload, &result).await;

                match result {
                    PushResult::Sent => {
                        let _ = subscription_repo::update_last_used(&state.db, sub.id).await;
                        SendResultItem { subscription_id: sub.id, status: "sent".into(), error: None }
                    }
                    PushResult::Gone => SendResultItem {
                        subscription_id: sub.id,
                        status: "gone".into(),
                        error: None,
                    },
                    PushResult::Failed(e) => SendResultItem {
                        subscription_id: sub.id,
                        status: "failed".into(),
                        error: Some(e),
                    },
                }
            }
        })
        .buffer_unordered(10)
        .collect()
        .await;

    let sent = results.iter().filter(|r| r.status == "sent").count();
    let gone = results.iter().filter(|r| r.status == "gone").count();
    let failed = results.iter().filter(|r| r.status == "failed").count();

    Ok(SendResult { total, sent, failed, gone, results })
}

pub async fn broadcast(
    state: &AppState,
    tags: &[String],
    payload: &Value,
    ttl: u32,
) -> anyhow::Result<BroadcastResult> {
    let subscriptions = subscription_repo::list_by_tags(&state.db, tags).await?;
    let total = subscriptions.len();

    let counts: Vec<&str> = stream::iter(subscriptions)
        .map(|sub| {
            let state = state.clone();
            let payload = payload.clone();
            async move {
                let result = vapid_service::send_push(
                    &sub.endpoint,
                    &sub.p256dh,
                    &sub.auth,
                    &payload,
                    &state.vapid_private_key_pem,
                    &state.config.vapid_contact,
                    ttl,
                )
                .await;

                log_result(&state, sub.id, &sub.endpoint, &payload, &result).await;

                match result {
                    PushResult::Sent => {
                        let _ = subscription_repo::update_last_used(&state.db, sub.id).await;
                        "sent"
                    }
                    PushResult::Gone => "gone",
                    PushResult::Failed(_) => "failed",
                }
            }
        })
        .buffer_unordered(10)
        .collect()
        .await;

    let sent = counts.iter().filter(|&&s| s == "sent").count();
    let gone = counts.iter().filter(|&&s| s == "gone").count();
    let failed = counts.iter().filter(|&&s| s == "failed").count();

    Ok(BroadcastResult { total, sent, failed, gone })
}

async fn log_result(
    state: &AppState,
    subscription_id: Uuid,
    endpoint: &str,
    payload: &Value,
    result: &PushResult,
) {
    let (status, status_code, error_message) = match result {
        PushResult::Sent => ("sent", Some(201i16), None),
        PushResult::Gone => ("gone", Some(410i16), None),
        PushResult::Failed(e) => ("failed", None, Some(e.as_str())),
    };

    if let Err(e) = notification_log_repo::create(
        &state.db,
        Some(subscription_id),
        endpoint,
        status,
        status_code,
        error_message,
        Some(payload.clone()),
    )
    .await
    {
        tracing::warn!("Failed to write notification log: {}", e);
    }
}
