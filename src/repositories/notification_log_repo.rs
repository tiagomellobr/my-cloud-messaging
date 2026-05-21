use crate::models::notification::{ListLogsQuery, NotificationLog};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(
    pool: &PgPool,
    subscription_id: Option<Uuid>,
    endpoint: &str,
    status: &str,
    status_code: Option<i16>,
    error_message: Option<&str>,
    payload: Option<serde_json::Value>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO notification_logs
           (subscription_id, endpoint, status, status_code, error_message, payload, sent_at)
           VALUES ($1, $2, $3::notification_status, $4, $5, $6, NOW())"#,
    )
    .bind(subscription_id)
    .bind(endpoint)
    .bind(status)
    .bind(status_code)
    .bind(error_message)
    .bind(payload)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list(
    pool: &PgPool,
    query: &ListLogsQuery,
) -> Result<Vec<NotificationLog>, sqlx::Error> {
    let per_page = query.per_page.unwrap_or(50).min(100);
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) * per_page;

    if let Some(status) = &query.status {
        sqlx::query_as::<_, NotificationLog>(
            r#"SELECT id, subscription_id, endpoint, status::TEXT, status_code,
                      error_message, payload, sent_at, created_at
               FROM notification_logs WHERE status = $1::notification_status
               ORDER BY created_at DESC LIMIT $2 OFFSET $3"#,
        )
        .bind(status)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, NotificationLog>(
            r#"SELECT id, subscription_id, endpoint, status::TEXT, status_code,
                      error_message, payload, sent_at, created_at
               FROM notification_logs ORDER BY created_at DESC LIMIT $1 OFFSET $2"#,
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await
    }
}
