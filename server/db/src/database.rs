// rustimport:pyo3

use crate::{error::DatabaseError, models, queries};
use pyo3::{exceptions::PyValueError, prelude::*};
use rusqlite::{Connection, Row};
use std::sync::{Arc, Mutex};

pub trait Query {
    type ResultType;
    fn query(&self) -> &str;
    fn params(&self) -> Vec<&dyn rusqlite::ToSql>;
    fn map_result(&self, row: &Row) -> Result<Self::ResultType, rusqlite::Error>;
    fn after_query(&self, _conn: &Connection) -> Result<(), rusqlite::Error> {
        Ok(())
    }
}

pub trait Database {
    fn execute<T: Query>(&self, query: T) -> Result<T::ResultType, DatabaseError>;
}

#[pyclass]
pub struct DatabaseManager {
    connection: Arc<Mutex<Connection>>,
}

impl Database for DatabaseManager {
    fn execute<T: Query>(&self, query: T) -> Result<T::ResultType, DatabaseError> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(query.query())?;
        let params = query.params();

        let result = stmt.query_row(&*params, |row| query.map_result(row));

        match result {
            Ok(value) => {
                query.after_query(&conn)?;
                Ok(value)
            }
            Err(e) => Err(e.into()),
        }
    }
}

#[pymethods]
impl DatabaseManager {
    #[new]
    pub fn new(db_path: &str) -> PyResult<Self> {
        let connection =
            Connection::open(db_path).map_err(|e| PyValueError::new_err(e.to_string()))?;

        let manager = DatabaseManager {
            connection: Arc::new(Mutex::new(connection)),
        };
        manager.init_db()?;
        Ok(manager)
    }

    fn init_db(&self) -> PyResult<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS oauth_tokens (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                access_token TEXT UNIQUE NOT NULL,
                access_secret TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                last_used TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS oauth_states (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                request_token TEXT NOT NULL,
                request_secret TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(())
    }

    pub fn get_oauth_state(&self, state: &str) -> PyResult<models::OAuthState> {
        let query = queries::GetOAuthState::new(state.to_string());
        match self.execute(query) {
            Ok(state) => Ok(state),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    pub fn store_oauth_state(&self, request_token: &str, request_secret: &str) -> PyResult<bool> {
        let query =
            queries::StoreOAuthState::new(request_token.to_string(), request_secret.to_string());
        match self.execute(query) {
            Ok(result) => Ok(result),
            Err(DatabaseError::UniqueViolation) => Ok(false),
            Err(DatabaseError::NotFound) => Ok(false),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    pub fn delete_oauth_state(&self, state: &str) -> PyResult<bool> {
        let query = queries::DeleteOAuthState::new(state.to_string());
        match self.execute(query) {
            Ok(result) => Ok(result),
            Err(DatabaseError::NotFound) => Ok(false),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    pub fn get_tokens(&self, access_token: &str) -> PyResult<Option<(String, String)>> {
        let query = queries::GetTokens::new(access_token.to_string());
        match self.execute(query) {
            Ok(Some(token_pair)) => Ok(Some((token_pair.access_token, token_pair.access_secret))),
            Ok(None) => Ok(None),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }
    pub fn store_tokens(&self, access_token: &str, access_secret: &str) -> PyResult<bool> {
        let query = queries::StoreTokens::new(access_token.to_string(), access_secret.to_string());
        match self.execute(query) {
            Ok(result) => Ok(result),
            Err(DatabaseError::UniqueViolation) => Ok(false),
            Err(DatabaseError::NotFound) => Ok(false),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    pub fn delete_tokens(&self, access_token: &str) -> PyResult<bool> {
        let query = queries::DeleteTokens::new(access_token.to_string());
        match self.execute(query) {
            Ok(result) => Ok(result),
            Err(DatabaseError::NotFound) => Ok(false),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }
}
