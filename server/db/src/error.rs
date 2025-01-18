// rustimport:pyo3

use rusqlite;
use std::fmt;

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
