use sqlx::SqlitePool;

/// Insert a new user.
pub struct Create<'a> {
    pub username: &'a str,
    pub password_hash: &'a str,
    pub email: &'a str,
    pub first_name: &'a str,
    pub last_name: &'a str,
}

impl Create<'_> {
    /// # Errors
    ///
    /// Returns an error if the insert fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
            "INSERT INTO users (username, password_hash, email, first_name, last_name) VALUES (?, ?, ?, ?, ?)",
            self.username,
            self.password_hash,
            self.email,
            self.first_name,
            self.last_name,
        )
        .execute(pool)
        .await?;
        Ok(result.last_insert_rowid())
    }
}

/// Database row for a user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub email: String,
    pub email_verified: bool,
    pub email_verified_at: Option<String>,
    pub first_name: String,
    pub last_name: String,
}

/// Fetch a user by username.
pub struct GetByUsername<'a> {
    pub username: &'a str,
}

impl GetByUsername<'_> {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Option<Row>, sqlx::Error> {
        sqlx::query_as!(
            Row,
            r#"SELECT
                id as "id!: i64",
                username,
                password_hash,
                email,
                email_verified as "email_verified!: bool",
                email_verified_at,
                first_name,
                last_name
            FROM users WHERE username = ?"#,
            self.username,
        )
        .fetch_optional(pool)
        .await
    }
}

/// Fetch a user by ID.
pub struct GetById {
    pub id: i64,
}

impl GetById {
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<Option<Row>, sqlx::Error> {
        sqlx::query_as!(
            Row,
            r#"SELECT
                id as "id!: i64",
                username,
                password_hash,
                email,
                email_verified as "email_verified!: bool",
                email_verified_at,
                first_name,
                last_name
            FROM users WHERE id = ?"#,
            self.id,
        )
        .fetch_optional(pool)
        .await
    }
}

/// Update a user's email address and reset verification status.
pub struct UpdateEmail<'a> {
    pub user_id: i64,
    pub email: &'a str,
}

impl UpdateEmail<'_> {
    /// # Errors
    ///
    /// Returns an error if the update fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE users SET email = ?, email_verified = 0, email_verified_at = NULL WHERE id = ?",
            self.email,
            self.user_id,
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

/// Update a user's profile (first name and last name).
pub struct UpdateProfile<'a> {
    pub user_id: i64,
    pub first_name: &'a str,
    pub last_name: &'a str,
}

impl UpdateProfile<'_> {
    /// # Errors
    ///
    /// Returns an error if the update fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE users SET first_name = ?, last_name = ? WHERE id = ?",
            self.first_name,
            self.last_name,
            self.user_id,
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

/// Mark a user's email as verified.
pub struct SetEmailVerified {
    pub user_id: i64,
}

impl SetEmailVerified {
    /// # Errors
    ///
    /// Returns an error if the update fails.
    pub async fn execute(&self, pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE users SET email_verified = 1, email_verified_at = datetime('now') WHERE id = ?",
            self.user_id,
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::tests::test_pool;

    #[tokio::test]
    async fn user_and_session_crud() {
        let pool = test_pool().await;
        let user_id = Create {
            username: "alice",
            password_hash: "hash",
            email: "",
            first_name: "",
            last_name: "",
        }
        .execute(&pool)
        .await
        .expect("user create failed");

        let user = GetByUsername { username: "alice" }
            .execute(&pool)
            .await
            .expect("user lookup failed")
            .expect("user should exist");
        assert_eq!(user.id, user_id);
        assert!(user.email.is_empty());
        assert!(!user.email_verified);

        crate::db::sessions::Create {
            token: "session-token",
            user_id,
            expires_at: "2999-01-01 00:00:00",
        }
        .execute(&pool)
        .await
        .expect("session create failed");

        let session = (crate::db::sessions::Get {
            token: "session-token",
        })
        .execute(&pool)
        .await
        .expect("session lookup failed")
        .expect("session should exist");
        assert_eq!(session.user_id, user_id);

        (crate::db::sessions::Delete {
            token: "session-token",
        })
        .execute(&pool)
        .await
        .expect("session delete failed");
        let deleted = (crate::db::sessions::Get {
            token: "session-token",
        })
        .execute(&pool)
        .await
        .expect("session lookup after delete failed");
        assert!(deleted.is_none());
    }

    #[tokio::test]
    async fn user_with_email() {
        let pool = test_pool().await;
        let user_id = Create {
            username: "bob",
            password_hash: "hash",
            email: "bob@example.com",
            first_name: "Bob",
            last_name: "Smith",
        }
        .execute(&pool)
        .await
        .expect("user create failed");

        let user = GetById { id: user_id }
            .execute(&pool)
            .await
            .expect("user lookup failed")
            .expect("user should exist");
        assert_eq!(user.email, "bob@example.com");
        assert!(!user.email_verified);
        assert!(user.email_verified_at.is_none());

        SetEmailVerified { user_id }
            .execute(&pool)
            .await
            .expect("verify failed");

        let verified = GetById { id: user_id }
            .execute(&pool)
            .await
            .expect("lookup failed")
            .expect("user should exist");
        assert!(verified.email_verified);
        assert!(verified.email_verified_at.is_some());
    }

    #[tokio::test]
    async fn update_email_resets_verification() {
        let pool = test_pool().await;
        let user_id = Create {
            username: "carol",
            password_hash: "hash",
            email: "carol@example.com",
            first_name: "",
            last_name: "",
        }
        .execute(&pool)
        .await
        .expect("create failed");

        SetEmailVerified { user_id }
            .execute(&pool)
            .await
            .expect("verify failed");

        UpdateEmail {
            user_id,
            email: "new@example.com",
        }
        .execute(&pool)
        .await
        .expect("update failed");

        let user = GetById { id: user_id }
            .execute(&pool)
            .await
            .expect("lookup failed")
            .expect("user should exist");
        assert_eq!(user.email, "new@example.com");
        assert!(!user.email_verified);
        assert!(user.email_verified_at.is_none());
    }

    #[tokio::test]
    async fn duplicate_email_rejected() {
        let pool = test_pool().await;
        Create {
            username: "alice",
            password_hash: "hash",
            email: "shared@example.com",
            first_name: "",
            last_name: "",
        }
        .execute(&pool)
        .await
        .expect("first create failed");

        let result = Create {
            username: "bob",
            password_hash: "hash",
            email: "SHARED@example.com",
            first_name: "",
            last_name: "",
        }
        .execute(&pool)
        .await;
        assert!(result.is_err());
    }
}
