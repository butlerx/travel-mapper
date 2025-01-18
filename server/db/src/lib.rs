// rustimport:pyo3

use pyo3::prelude::*;

mod database;
mod error;
mod models;
mod queries;

pub use database::{Database, DatabaseManager};
pub use error::DatabaseError;
pub use models::*;
pub use queries::*;

#[pymodule]
fn db(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<DatabaseManager>()?;
    m.add_class::<OAuthState>()?;
    Ok(())
}
