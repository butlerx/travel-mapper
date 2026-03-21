use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Serialize, JsonSchema)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct StatusResponse {
    pub status: String,
}
