use crate::models::subscription::{ListSubscriptionsQuery, Subscription};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(
    pool: &PgPool,
    endpoint: &str,
    p256dh: &str,
    auth: &str,
    tags: &[String],
    metadata: Option<serde_json::Value>,
) -> Result<Subscription, sqlx::Error> {
    sqlx::query_as::<_, Subscription>(
        r#"
        INSERT INTO subscriptions (endpoint, p256dh, auth, tags, metadata)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, endpoint, p256dh, auth, tags, metadata, created_at, last_used_at
        "#,
    )
    .bind(endpoint)
    .bind(p256dh)
    .bind(auth)
    .bind(tags)
    .bind(metadata)
    .fetch_one(pool)
    .await
}

pub async fn find_by_endpoint(
    pool: &PgPool,
    endpoint: &str,
) -> Result<Option<Subscription>, sqlx::Error> {
    sqlx::query_as::<_, Subscription>(
        "SELECT id, endpoint, p256dh, auth, tags, metadata, created_at, last_used_at
         FROM subscriptions WHERE endpoint = $1",
    )
    .bind(endpoint)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Subscription>, sqlx::Error> {
    sqlx::query_as::<_, Subscription>(
        "SELECT id, endpoint, p256dh, auth, tags, metadata, created_at, last_used_at
         FROM subscriptions WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn find_by_ids(pool: &PgPool, ids: &[Uuid]) -> Result<Vec<Subscription>, sqlx::Error> {
    sqlx::query_as::<_, Subscription>(
        "SELECT id, endpoint, p256dh, auth, tags, metadata, created_at, last_used_at
         FROM subscriptions WHERE id = ANY($1)",
    )
    .bind(ids)
    .fetch_all(pool)
    .await
}

pub async fn list(
    pool: &PgPool,
    query: &ListSubscriptionsQuery,
) -> Result<Vec<Subscription>, sqlx::Error> {
    let per_page = query.per_page.unwrap_or(50).min(100);
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) * per_page;

    if let Some(tag) = &query.tag {
        sqlx::query_as::<_, Subscription>(
            "SELECT id, endpoint, p256dh, auth, tags, metadata, created_at, last_used_at
             FROM subscriptions WHERE $1 = ANY(tags)
             ORDER BY created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(tag)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, Subscription>(
            "SELECT id, endpoint, p256dh, auth, tags, metadata, created_at, last_used_at
             FROM subscriptions ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await
    }
}

pub async fn list_by_tags(
    pool: &PgPool,
    tags: &[String],
) -> Result<Vec<Subscription>, sqlx::Error> {
    if tags.is_empty() {
        sqlx::query_as::<_, Subscription>(
            "SELECT id, endpoint, p256dh, auth, tags, metadata, created_at, last_used_at
             FROM subscriptions ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, Subscription>(
            "SELECT id, endpoint, p256dh, auth, tags, metadata, created_at, last_used_at
             FROM subscriptions WHERE tags && $1 ORDER BY created_at DESC",
        )
        .bind(tags)
        .fetch_all(pool)
        .await
    }
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM subscriptions WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn delete_by_endpoint(pool: &PgPool, endpoint: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM subscriptions WHERE endpoint = $1")
        .bind(endpoint)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_last_used(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE subscriptions SET last_used_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn cleanup_gone(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"DELETE FROM subscriptions
           WHERE id IN (
               SELECT DISTINCT subscription_id FROM notification_logs
               WHERE status = 'gone' AND subscription_id IS NOT NULL
           )"#,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}
