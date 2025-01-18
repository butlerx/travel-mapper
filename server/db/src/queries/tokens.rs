// rustimport:pyo3

use crate::{database::Query, models::TokenPair};
use rusqlite::{Connection, Error, Row};

pub struct GetTokens {
    access_token: String,
}

impl GetTokens {
    pub fn new(access_token: String) -> Self {
        Self { access_token }
    }
}

impl Query for GetTokens {
    type ResultType = Option<TokenPair>;

    fn query(&self) -> &str {
        "SELECT access_token, access_secret FROM oauth_tokens WHERE access_token = ?1"
    }

    fn params(&self) -> Vec<&dyn rusqlite::ToSql> {
        vec![&self.access_token]
    }

    fn map_result(&self, row: &Row) -> Result<Self::ResultType, Error> {
        Ok(Some(TokenPair {
            access_token: row.get(0)?,
            access_secret: row.get(1)?,
        }))
    }

    fn after_query(&self, conn: &Connection) -> Result<(), Error> {
        conn.execute(
            "UPDATE oauth_tokens SET last_used = CURRENT_TIMESTAMP WHERE access_token = ?1",
            [&self.access_token],
        )?;
        Ok(())
    }
}

pub struct StoreTokens {
    access_token: String,
    access_secret: String,
}

impl StoreTokens {
    pub fn new(access_token: String, access_secret: String) -> Self {
        Self {
            access_token,
            access_secret,
        }
    }
}

impl Query for StoreTokens {
    type ResultType = bool;

    fn query(&self) -> &str {
        "INSERT INTO oauth_tokens (access_token, access_secret) VALUES (?1, ?2)"
    }

    fn params(&self) -> Vec<&dyn rusqlite::ToSql> {
        vec![&self.access_token, &self.access_secret]
    }

    fn map_result(&self, _: &Row) -> Result<Self::ResultType, Error> {
        Ok(true)
    }
}

pub struct DeleteTokens {
    access_token: String,
}

impl DeleteTokens {
    pub fn new(access_token: String) -> Self {
        Self { access_token }
    }
}

impl Query for DeleteTokens {
    type ResultType = bool;

    fn query(&self) -> &str {
        "DELETE FROM oauth_tokens WHERE access_token = ?1"
    }

    fn params(&self) -> Vec<&dyn rusqlite::ToSql> {
        vec![&self.access_token]
    }

    fn map_result(&self, _: &Row) -> Result<Self::ResultType, Error> {
        Ok(true)
    }
}
