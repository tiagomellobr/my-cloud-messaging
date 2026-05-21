use anyhow::Context;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use p256::{
    elliptic_curve::sec1::ToEncodedPoint,
    pkcs8::{EncodePrivateKey, LineEnding},
    SecretKey,
};
use rand::rngs::OsRng;
use sqlx::PgPool;
use web_push::{
    ContentEncoding, IsahcWebPushClient, SubscriptionInfo, VapidSignatureBuilder,
    WebPushClient, WebPushError, WebPushMessageBuilder,
};

pub struct VapidKeys {
    pub private_key_pem: String,
    pub public_key_b64: String,
}

pub async fn init(pool: &PgPool) -> anyhow::Result<VapidKeys> {
    let row = sqlx::query_as::<_, (String, String)>(
        "SELECT private_key_pem, public_key_b64 FROM vapid_keys WHERE id = 1",
    )
    .fetch_optional(pool)
    .await
    .context("Failed to query vapid_keys")?;

    if let Some((private_key_pem, public_key_b64)) = row {
        return Ok(VapidKeys { private_key_pem, public_key_b64 });
    }

    let keys = generate_keys()?;

    sqlx::query(
        "INSERT INTO vapid_keys (id, private_key_pem, public_key_b64) VALUES (1, $1, $2)",
    )
    .bind(&keys.private_key_pem)
    .bind(&keys.public_key_b64)
    .execute(pool)
    .await
    .context("Failed to store VAPID keys")?;

    tracing::info!(
        public_key = %keys.public_key_b64,
        "VAPID keys generated and stored. Use this public key in your JS client applicationServerKey."
    );

    Ok(keys)
}

fn generate_keys() -> anyhow::Result<VapidKeys> {
    let secret_key = SecretKey::random(&mut OsRng);

    let private_key_pem = secret_key
        .to_pkcs8_pem(LineEnding::LF)
        .context("Failed to encode private key as PEM")?
        .to_string();

    let public_point = secret_key.public_key().to_encoded_point(false);
    let public_key_b64 = URL_SAFE_NO_PAD.encode(public_point.as_bytes());

    Ok(VapidKeys {
        private_key_pem,
        public_key_b64,
    })
}

pub enum PushResult {
    Sent,
    Gone,
    Failed(String),
}

pub async fn send_push(
    endpoint: &str,
    p256dh: &str,
    auth: &str,
    payload: &serde_json::Value,
    private_key_pem: &str,
    contact: &str,
    ttl: u32,
) -> PushResult {
    match try_send(endpoint, p256dh, auth, payload, private_key_pem, contact, ttl).await {
        Ok(_) => PushResult::Sent,
        Err(WebPushError::EndpointNotFound(_)) => PushResult::Gone,
        Err(WebPushError::EndpointNotValid(_)) => PushResult::Gone,
        Err(e) => PushResult::Failed(e.to_string()),
    }
}

async fn try_send(
    endpoint: &str,
    p256dh: &str,
    auth: &str,
    payload: &serde_json::Value,
    private_key_pem: &str,
    contact: &str,
    ttl: u32,
) -> Result<(), WebPushError> {
    // SubscriptionInfo::new and WebPushMessageBuilder::new are infallible in 0.11
    let subscription_info = SubscriptionInfo::new(endpoint, p256dh, auth);

    let mut sig_builder =
        VapidSignatureBuilder::from_pem(private_key_pem.as_bytes(), &subscription_info)?;
    sig_builder.add_claim("sub", contact);
    let vapid_signature = sig_builder.build()?;

    // Value always serializes successfully
    let payload_bytes = serde_json::to_vec(payload).unwrap();

    let mut builder = WebPushMessageBuilder::new(&subscription_info);
    builder.set_payload(ContentEncoding::Aes128Gcm, &payload_bytes);
    builder.set_vapid_signature(vapid_signature);
    builder.set_ttl(ttl);
    let message = builder.build()?;

    let client = IsahcWebPushClient::new()?;
    client.send(message).await?;

    Ok(())
}
