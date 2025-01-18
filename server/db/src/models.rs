// rustimport:pyo3

use pyo3::prelude::*;

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
