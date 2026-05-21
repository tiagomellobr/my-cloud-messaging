use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NotificationPayload {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
    pub url: Option<String>,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct SendNotificationRequest {
    pub subscription_ids: Vec<Uuid>,
    pub notification: NotificationPayload,
    pub ttl: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct BroadcastNotificationRequest {
    #[serde(default)]
    pub tags: Vec<String>,
    pub notification: NotificationPayload,
    pub ttl: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct SendResult {
    pub total: usize,
    pub sent: usize,
    pub failed: usize,
    pub gone: usize,
    pub results: Vec<SendResultItem>,
}

#[derive(Debug, Serialize)]
pub struct SendResultItem {
    pub subscription_id: Uuid,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BroadcastResult {
    pub total: usize,
    pub sent: usize,
    pub failed: usize,
    pub gone: usize,
}

#[derive(Debug, Serialize, FromRow)]
pub struct NotificationLog {
    pub id: Uuid,
    pub subscription_id: Option<Uuid>,
    pub endpoint: String,
    pub status: String,
    pub status_code: Option<i16>,
    pub error_message: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub sent_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

use sqlx::FromRow;

#[derive(Debug, Deserialize)]
pub struct ListLogsQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub status: Option<String>,
}
