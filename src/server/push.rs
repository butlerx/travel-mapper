//! Web Push notification delivery.

use crate::db;
use serde_json::json;
use sqlx::SqlitePool;
use web_push::{
    ContentEncoding, IsahcWebPushClient, SubscriptionInfo, VapidSignatureBuilder, WebPushClient,
    WebPushError, WebPushMessageBuilder,
};

pub(crate) async fn send_to_user(
    pool: &SqlitePool,
    vapid_key: &[u8],
    user_id: i64,
    title: &str,
    body: &str,
    url: &str,
) -> () {
    let subscriptions = match (db::push_subscriptions::GetByUserId { user_id })
        .execute(pool)
        .await
    {
        Ok(rows) => rows,
        Err(err) => {
            tracing::warn!(user_id, error = %err, "failed to load push subscriptions");
            return;
        }
    };

    if subscriptions.is_empty() {
        return;
    }

    let client = match IsahcWebPushClient::new() {
        Ok(client) => client,
        Err(err) => {
            tracing::warn!(user_id, error = %err, "failed to create web push client");
            return;
        }
    };

    for subscription in subscriptions {
        let payload_value = json!({
            "title": title,
            "body": body,
            "url": url,
        });
        let payload = match serde_json::to_vec(&payload_value) {
            Ok(payload) => payload,
            Err(err) => {
                tracing::warn!(user_id, error = %err, "failed to serialize push payload");
                continue;
            }
        };

        if payload.len() > 3_800 {
            tracing::warn!(
                user_id,
                endpoint = subscription.endpoint,
                payload_bytes = payload.len(),
                "push payload too large, skipping notification"
            );
            continue;
        }

        let info = SubscriptionInfo::new(
            subscription.endpoint.clone(),
            subscription.p256dh,
            subscription.auth,
        );
        let mut builder = WebPushMessageBuilder::new(&info);
        builder.set_payload(ContentEncoding::Aes128Gcm, &payload);

        let signature = match VapidSignatureBuilder::from_pem_no_sub(vapid_key)
            .map(|partial| partial.add_sub_info(&info))
            .and_then(VapidSignatureBuilder::build)
        {
            Ok(signature) => signature,
            Err(err) => {
                tracing::warn!(user_id, endpoint = info.endpoint, error = %err, "failed to build VAPID signature");
                continue;
            }
        };

        builder.set_vapid_signature(signature);

        let message = match builder.build() {
            Ok(message) => message,
            Err(err) => {
                tracing::warn!(user_id, endpoint = info.endpoint, error = %err, "failed to build push message");
                continue;
            }
        };

        if let Err(err) = client.send(message).await {
            match err {
                WebPushError::EndpointNotValid(_) | WebPushError::EndpointNotFound(_) => {
                    if let Err(delete_err) = (db::push_subscriptions::DeleteByEndpoint {
                        endpoint: &info.endpoint,
                    })
                    .execute(pool)
                    .await
                    {
                        tracing::warn!(
                            user_id,
                            endpoint = info.endpoint,
                            error = %delete_err,
                            "failed to delete stale push subscription"
                        );
                    }
                }
                other => {
                    tracing::warn!(user_id, endpoint = info.endpoint, error = %other, "failed to send push notification");
                }
            }
        }
    }
}
