// rustimport:pyo3

//:
//: [dependencies]
//: rusqlite = "0.32"

use pyo3::{exceptions::PyValueError, prelude::*};
use rusqlite::{Connection, Error, Row};
use std::{
    fmt,
    sync::{Arc, Mutex},
};

#[pyclass]
#[derive(Debug)]
pub struct OAuthState {
    #[pyo3(get)]
    pub request_token: String,
    #[pyo3(get)]
    pub request_secret: String,
    #[pyo3(get)]
    pub timestamp: f64,
}

#[pymethods]
impl OAuthState {
    #[new]
    pub fn new(request_token: String, request_secret: String, timestamp: f64) -> Self {
        Self {
            request_token,
            request_secret,
            timestamp,
        }
    }
}

#[derive(Debug)]
pub struct TokenPair {
    pub access_token: String,
    pub access_secret: String,
}

#[derive(Debug)]
pub enum DatabaseError {
    UniqueViolation,
    NotFound,
    Other(String),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DatabaseError::UniqueViolation => write!(f, "Unique violation"),
            DatabaseError::NotFound => write!(f, "Entry Not found"),
            DatabaseError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<rusqlite::Error> for DatabaseError {
    fn from(error: rusqlite::Error) -> Self {
        match error {
            rusqlite::Error::SqliteFailure(_, Some(msg))
                if msg.contains("UNIQUE constraint failed") =>
            {
                DatabaseError::UniqueViolation
            }
            rusqlite::Error::QueryReturnedNoRows => DatabaseError::NotFound,
            e => DatabaseError::Other(e.to_string()),
        }
    }
}

pub trait Query {
    type ResultType;
    fn query(&self) -> &str;
    fn bind_params(&self, stmt: &mut rusqlite::Statement) -> Result<(), Error>;
    fn map_result(&self, row: &Row) -> Result<Self::ResultType, Error>;
    fn after_query(&self, _conn: &Connection) -> Result<(), Error> {
        Ok(())
    }
}

pub trait Database {
    fn execute<T: Query>(&self, query: T) -> Result<T::ResultType, DatabaseError>;
}

#[pyclass]
pub struct DatabaseManager {
    db_path: String,
    connection: Arc<Mutex<Connection>>,
}

impl Database for DatabaseManager {
    fn execute<T: Query>(&self, query: T) -> Result<T::ResultType, DatabaseError> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(query.query())?;
        query.bind_params(&mut stmt)?;

        let result = stmt.query_row([], |row| query.map_result(row));

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
            db_path: db_path.to_string(),
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

    pub fn get_oauth_state(&self, state: &str) -> PyResult<OAuthState> {
        let query = GetOAuthState::new(state.to_string());
        match self.execute(query) {
            Ok(state) => Ok(state),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    pub fn store_oauth_state(&self, request_token: &str, request_secret: &str) -> PyResult<bool> {
        let query = StoreOAuthState {
            request_token: request_token.to_string(),
            request_secret: request_secret.to_string(),
        };
        match self.execute(query) {
            Ok(result) => Ok(result),
            Err(DatabaseError::UniqueViolation) => Ok(false),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    pub fn delete_oauth_state(&self, state: &str) -> PyResult<bool> {
        let query = DeleteOAuthState::new(state.to_string());
        self.execute(query)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    pub fn get_tokens(&self, access_token: &str) -> PyResult<Option<(String, String)>> {
        let query = GetTokens::new(access_token.to_string());
        match self.execute(query) {
            Ok(Some(token_pair)) => Ok(Some((token_pair.access_token, token_pair.access_secret))),
            Ok(None) => Ok(None),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }
    pub fn store_tokens(&self, access_token: &str, access_secret: &str) -> PyResult<bool> {
        let query = StoreTokens::new(access_token.to_string(), access_secret.to_string());
        match self.execute(query) {
            Ok(result) => Ok(result),
            Err(DatabaseError::UniqueViolation) => Ok(false),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    pub fn delete_tokens(&self, access_token: &str) -> PyResult<bool> {
        let query = DeleteTokens::new(access_token.to_string());
        self.execute(query)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }
}

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

    fn bind_params(&self, stmt: &mut rusqlite::Statement) -> Result<(), Error> {
        stmt.raw_bind_parameter(1, &self.state)
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

    fn bind_params(&self, stmt: &mut rusqlite::Statement) -> Result<(), Error> {
        stmt.raw_bind_parameter(1, &self.request_token)?;
        stmt.raw_bind_parameter(2, &self.request_secret)?;
        Ok(())
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

    fn bind_params(&self, stmt: &mut rusqlite::Statement) -> Result<(), Error> {
        stmt.raw_bind_parameter(1, &self.state)
    }

    fn map_result(&self, _: &Row) -> Result<Self::ResultType, Error> {
        Ok(true)
    }
}

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

    fn bind_params(&self, stmt: &mut rusqlite::Statement) -> Result<(), Error> {
        stmt.raw_bind_parameter(1, &self.access_token)
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

    fn bind_params(&self, stmt: &mut rusqlite::Statement) -> Result<(), Error> {
        stmt.raw_bind_parameter(1, &self.access_token)?;
        stmt.raw_bind_parameter(2, &self.access_secret)?;
        Ok(())
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

    fn bind_params(&self, stmt: &mut rusqlite::Statement) -> Result<(), Error> {
        stmt.raw_bind_parameter(1, &self.access_token)
    }

    fn map_result(&self, _: &Row) -> Result<Self::ResultType, Error> {
        Ok(true)
    }
}
