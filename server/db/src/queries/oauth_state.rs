// rustimport:pyo3

use crate::{database::Query, models::OAuthState};
use rusqlite::{Error, Row};

pub struct GetOAuthState {
    state: String,
}

impl GetOAuthState {
    pub fn new(state: String) -> Self {
        Self { state }
    }
}

impl Query for GetOAuthState {
    type ResultType = OAuthState;

    fn query(&self) -> &str {
        "SELECT
            request_token,
            request_secret,
            strftime('%s', created_at) as created_timestamp
         FROM oauth_states
         WHERE request_token = ?1"
    }

    fn params(&self) -> Vec<&dyn rusqlite::ToSql> {
        vec![&self.state]
    }

    fn map_result(&self, row: &Row) -> Result<Self::ResultType, Error> {
        let timestamp = row.get::<_, String>(2)?.parse::<f64>().unwrap();
        Ok(OAuthState {
            request_token: row.get(0)?,
            request_secret: row.get(1)?,
            timestamp,
        })
    }
}

pub struct StoreOAuthState {
    request_token: String,
    request_secret: String,
}

impl StoreOAuthState {
    pub fn new(request_token: String, request_secret: String) -> Self {
        Self {
            request_token,
            request_secret,
        }
    }
}

impl Query for StoreOAuthState {
    type ResultType = bool;

    fn query(&self) -> &str {
        "INSERT INTO oauth_states (request_token, request_secret) VALUES (?1, ?2)"
    }

    fn params(&self) -> Vec<&dyn rusqlite::ToSql> {
        vec![&self.request_token, &self.request_secret]
    }

    fn map_result(&self, _: &Row) -> Result<Self::ResultType, Error> {
        Ok(true)
    }
}

pub struct DeleteOAuthState {
    state: String,
}

impl DeleteOAuthState {
    pub fn new(state: String) -> Self {
        Self { state }
    }
}

impl Query for DeleteOAuthState {
    type ResultType = bool;

    fn query(&self) -> &str {
        "DELETE FROM oauth_states WHERE request_token = ?1"
    }

    fn params(&self) -> Vec<&dyn rusqlite::ToSql> {
        vec![&self.state]
    }

    fn map_result(&self, _: &Row) -> Result<Self::ResultType, Error> {
        Ok(true)
    }
}
