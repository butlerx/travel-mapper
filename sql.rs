// rustimport:pyo3

//:
//: [dependencies]
//: rusqlite = "0.32"

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use rusqlite::{Connection, Error};
use std::sync::Arc;
use std::sync::Mutex;

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

#[pyclass]
pub struct DatabaseManager {
    db_path: String,
    connection: Arc<Mutex<Connection>>,
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

    pub fn store_oauth_state(&self, request_token: &str, request_secret: &str) -> PyResult<bool> {
        let conn = self.connection.lock().unwrap();
        match conn.execute(
            "INSERT INTO oauth_states (request_token, request_secret)
             VALUES (?1, ?2)",
            [request_token, request_secret],
        ) {
            Ok(_) => Ok(true),
            Err(Error::SqliteFailure(_, Some(msg))) if msg.contains("UNIQUE constraint failed") => {
                Ok(false)
            }
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    pub fn get_oauth_state(&self, state: &str) -> PyResult<OAuthState> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT
                    request_token,
                    request_secret,
                    strftime('%s', created_at) as created_timestamp
                 FROM oauth_states WHERE request_token = ?1",
            )
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        let result = stmt.query_row([state], |row| {
            let timestamp = row.get::<_, String>(2)?.parse::<f64>().unwrap();
            Ok(OAuthState {
                request_token: row.get(0)?,
                request_secret: row.get(1)?,
                timestamp,
            })
        });

        match result {
            Ok(state) => Ok(state),
            Err(Error::QueryReturnedNoRows) => Err(PyValueError::new_err("No such state")),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    pub fn delete_oauth_state(&self, state: &str) -> PyResult<bool> {
        let conn = self.connection.lock().unwrap();
        conn.execute("DELETE FROM oauth_states WHERE request_token = ?1", [state])
            .map(|affected| affected > 0)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    pub fn store_tokens(&self, access_token: &str, access_secret: &str) -> PyResult<bool> {
        let conn = self.connection.lock().unwrap();
        match conn.execute(
            "INSERT INTO oauth_tokens (access_token, access_secret) VALUES (?1, ?2)",
            [access_token, access_secret],
        ) {
            Ok(_) => Ok(true),
            Err(Error::SqliteFailure(_, Some(msg))) if msg.contains("UNIQUE constraint failed") => {
                Ok(false)
            }
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    pub fn get_tokens(&self, access_token: &str) -> PyResult<Option<(String, String)>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT access_token, access_secret FROM oauth_tokens WHERE access_token = ?1")
            .map_err(|e| PyValueError::new_err(e.to_string()))?;

        let result = stmt.query_row([access_token], |row| Ok((row.get(0)?, row.get(1)?)));

        match result {
            Ok(tokens) => {
                conn.execute(
                    "UPDATE oauth_tokens SET last_used = CURRENT_TIMESTAMP WHERE access_token = ?1",
                    [access_token],
                )
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
                Ok(Some(tokens))
            }
            Err(Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }

    pub fn delete_tokens(&self, access_token: &str) -> PyResult<bool> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "DELETE FROM oauth_tokens WHERE access_token = ?1",
            [access_token],
        )
        .map(|affected| affected > 0)
        .map_err(|e| PyValueError::new_err(e.to_string()))
    }
}
