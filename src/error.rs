use thiserror::Error;

/// Errors returned by REST client operations.
#[derive(Debug, Error)]
pub enum ClientError {
    /// Base URL is not a valid absolute URL.
    #[error("invalid base URL '{0}'")]
    InvalidBaseUrl(String),

    /// Endpoint path could not be joined to the base URL.
    #[error("invalid endpoint path '{0}'")]
    InvalidPath(String),

    /// The requested `OpenAPI` operation id is not present in the generated catalog.
    #[error("unknown OpenAPI operation '{0}'")]
    UnknownOperation(String),

    /// A required path template parameter was not provided.
    #[error("missing required path parameter '{parameter}' for operation '{operation_id}'")]
    MissingPathParameter {
        operation_id: String,
        parameter: String,
    },

    /// HTTP transport-layer request failure.
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),

    /// Response body could not be parsed as JSON.
    #[error("failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),

    /// Non-success HTTP status with response payload.
    #[error("server returned status {status}: {body}")]
    HttpStatus {
        status: reqwest::StatusCode,
        body: String,
    },
}
