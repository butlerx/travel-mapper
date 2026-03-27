use sqlx::SqlitePool;

pub struct Row {
    pub id: i64,
    pub user_id: i64,
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
    pub created_at: String,
}

pub struct Create<'a> {
    pub user_id: i64,
    pub endpoint: &'a str,
    pub p256dh: &'a str,
    pub auth: &'a str,
}

impl Create<'_> {
    /// # Errors
    ///
    /// Returns an error if the insert fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
            "INSERT OR REPLACE INTO push_subscriptions (user_id, endpoint, p256dh, auth) VALUES (?, ?, ?, ?)",
            self.user_id,
            self.endpoint,
            self.p256dh,
            self.auth,
        )
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }
}

pub struct GetByUserId {
    pub user_id: i64,
}

impl GetByUserId {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Vec<Row>, sqlx::Error> {
        let rows = sqlx::query_as!(
            Row,
            r#"SELECT id as "id!: i64", user_id as "user_id!: i64", endpoint, p256dh, auth, created_at
               FROM push_subscriptions
               WHERE user_id = ?
               ORDER BY created_at DESC"#,
            self.user_id,
        )
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }
}

pub struct DeleteByUserAndEndpoint<'a> {
    pub user_id: i64,
    pub endpoint: &'a str,
}

impl DeleteByUserAndEndpoint<'_> {
    /// # Errors
    ///
    /// Returns an error if the delete fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM push_subscriptions WHERE user_id = ? AND endpoint = ?",
            self.user_id,
            self.endpoint,
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}

pub struct DeleteByEndpoint<'a> {
    pub endpoint: &'a str,
}

impl DeleteByEndpoint<'_> {
    /// # Errors
    ///
    /// Returns an error if the delete fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<bool, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM push_subscriptions WHERE endpoint = ?",
            self.endpoint,
        )
        .execute(pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests::{test_pool, test_user};

    #[tokio::test]
    async fn push_subscription_crud() {
        let pool = test_pool().await;
        let user_id = test_user(&pool, "alice").await;

        let row_id = Create {
            user_id,
            endpoint: "https://example.com/endpoint-1",
            p256dh: "p256dh-key",
            auth: "auth-key",
        }
        .execute(&pool)
        .await
        .expect("push subscription create failed");
        assert!(row_id > 0);

        let rows = GetByUserId { user_id }
            .execute(&pool)
            .await
            .expect("push subscription list failed");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].endpoint, "https://example.com/endpoint-1");
        assert_eq!(rows[0].p256dh, "p256dh-key");
        assert_eq!(rows[0].auth, "auth-key");

        let deleted = DeleteByUserAndEndpoint {
            user_id,
            endpoint: "https://example.com/endpoint-1",
        }
        .execute(&pool)
        .await
        .expect("push subscription delete failed");
        assert!(deleted);

        let rows_after = GetByUserId { user_id }
            .execute(&pool)
            .await
            .expect("push subscription list failed");
        assert!(rows_after.is_empty());
    }

    #[tokio::test]
    async fn push_subscription_delete_scoped_to_user() {
        let pool = test_pool().await;
        let alice_id = test_user(&pool, "alice").await;
        let bob_id = test_user(&pool, "bob").await;

        Create {
            user_id: alice_id,
            endpoint: "https://example.com/alice-endpoint",
            p256dh: "alice-p256dh",
            auth: "alice-auth",
        }
        .execute(&pool)
        .await
        .expect("create failed");

        let not_deleted = DeleteByUserAndEndpoint {
            user_id: bob_id,
            endpoint: "https://example.com/alice-endpoint",
        }
        .execute(&pool)
        .await
        .expect("delete failed");
        assert!(!not_deleted);

        let still_exists = GetByUserId { user_id: alice_id }
            .execute(&pool)
            .await
            .expect("list failed");
        assert_eq!(still_exists.len(), 1);
    }

    #[tokio::test]
    async fn delete_by_endpoint_removes_any_owner() {
        let pool = test_pool().await;
        let alice_id = test_user(&pool, "alice").await;

        Create {
            user_id: alice_id,
            endpoint: "https://example.com/shared-endpoint",
            p256dh: "p256dh",
            auth: "auth",
        }
        .execute(&pool)
        .await
        .expect("create failed");

        let deleted = DeleteByEndpoint {
            endpoint: "https://example.com/shared-endpoint",
        }
        .execute(&pool)
        .await
        .expect("delete by endpoint failed");
        assert!(deleted);
    }
}
