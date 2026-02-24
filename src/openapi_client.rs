use reqwest::Method;
use serde_json::Value;
use url::form_urlencoded::byte_serialize;

use crate::{ApiClient, BlockingApiClient, ClientError};

/// Metadata for one `OpenAPI` operation.
///
/// Values are generated from `openapi/openapi.json` at build time.
#[derive(Clone, Copy, Debug)]
pub struct OperationDefinition {
    /// Stable `OpenAPI` operation identifier.
    pub operation_id: &'static str,
    /// Uppercase HTTP method (for example `GET`, `POST`).
    pub method: &'static str,
    /// Path template, potentially containing `{param}` placeholders.
    pub path_template: &'static str,
    /// Required path parameter names extracted from `path_template`.
    pub path_params: &'static [&'static str],
}

// Generated file contract (`$OUT_DIR/openapi_operations.rs`):
// 1. `OPENAPI_DEFAULT_SERVER_URL: &str`
//    - Default base URL resolved from `openapi/openapi.json` (`servers[0].url`).
// 2. `OPENAPI_OPERATIONS: &[OperationDefinition]`
//    - One entry per OpenAPI operation with:
//      - `operation_id`
//      - `method` (uppercase)
//      - `path_template`
//      - `path_params`
//
// This contract is produced by `build.rs` and consumed by this module via `include!`.
include!(concat!(env!("OUT_DIR"), "/openapi_operations.rs"));

/// Async IRI API client backed by the `OpenAPI` operation registry.
///
/// Use this when you want to call endpoints via `operation_id` rather than
/// hard-coded URL paths.
#[derive(Clone, Debug)]
pub struct IriClient {
    inner: ApiClient,
}

impl IriClient {
    /// Creates a client with an explicit base URL.
    pub fn new(base_url: impl AsRef<str>) -> Result<Self, ClientError> {
        Ok(Self {
            inner: ApiClient::new(base_url)?,
        })
    }

    /// Creates a client using the first server URL from the `OpenAPI` spec.
    pub fn from_openapi_default_server() -> Result<Self, ClientError> {
        Self::new(openapi_default_server_url())
    }

    /// Returns a new client with a raw access token attached to all requests.
    ///
    /// This sets `Authorization: <token>` (without `Bearer ` prefix).
    #[must_use]
    pub fn with_authorization_token(mut self, token: impl Into<String>) -> Self {
        self.inner = self.inner.with_authorization_token(token);
        self
    }

    /// Returns all operations discovered from the `OpenAPI` spec.
    pub fn operations() -> &'static [OperationDefinition] {
        OPENAPI_OPERATIONS
    }

    /// Sends a request using a raw path and method.
    ///
    /// This bypasses operation-id lookup but keeps IRI client configuration.
    pub async fn request_json_with_query(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, &str)],
        body: Option<Value>,
    ) -> Result<Value, ClientError> {
        self.inner
            .request_json_with_query(method, path, query, body)
            .await
    }

    /// Calls an endpoint by `OpenAPI` `operation_id`.
    ///
    /// `path_params` replaces `{param}` segments in the operation path template.
    /// Missing required parameters return
    /// [`ClientError::MissingPathParameter`].
    pub async fn call_operation(
        &self,
        operation_id: &str,
        path_params: &[(&str, &str)],
        query: &[(&str, &str)],
        body: Option<Value>,
    ) -> Result<Value, ClientError> {
        let operation = find_operation(operation_id)?;
        let rendered_path = render_path(operation, path_params)?;
        let method = parse_method(operation)?;
        self.inner
            .request_json_with_query(method, &rendered_path, query, body)
            .await
    }
}

/// Blocking IRI API client backed by the `OpenAPI` operation registry.
///
/// This is the synchronous counterpart of [`IriClient`].
#[derive(Debug)]
pub struct BlockingIriClient {
    inner: BlockingApiClient,
}

impl BlockingIriClient {
    /// Creates a client with an explicit base URL.
    pub fn new(base_url: impl AsRef<str>) -> Result<Self, ClientError> {
        Ok(Self {
            inner: BlockingApiClient::new(base_url)?,
        })
    }

    /// Creates a client using the first server URL from the `OpenAPI` spec.
    pub fn from_openapi_default_server() -> Result<Self, ClientError> {
        Self::new(openapi_default_server_url())
    }

    /// Returns a new client with a raw access token attached to all requests.
    ///
    /// This sets `Authorization: <token>` (without `Bearer ` prefix).
    #[must_use]
    pub fn with_authorization_token(mut self, token: impl Into<String>) -> Self {
        self.inner = self.inner.with_authorization_token(token);
        self
    }

    /// Returns all operations discovered from the `OpenAPI` spec.
    pub fn operations() -> &'static [OperationDefinition] {
        OPENAPI_OPERATIONS
    }

    /// Sends a request using a raw path and method.
    ///
    /// This bypasses operation-id lookup but keeps IRI client configuration.
    pub fn request_json_with_query(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, &str)],
        body: Option<Value>,
    ) -> Result<Value, ClientError> {
        self.inner
            .request_json_with_query(method, path, query, body)
    }

    /// Calls an endpoint by `OpenAPI` `operation_id`.
    ///
    /// `path_params` replaces `{param}` segments in the operation path template.
    /// Missing required parameters return
    /// [`ClientError::MissingPathParameter`].
    pub fn call_operation(
        &self,
        operation_id: &str,
        path_params: &[(&str, &str)],
        query: &[(&str, &str)],
        body: Option<Value>,
    ) -> Result<Value, ClientError> {
        let operation = find_operation(operation_id)?;
        let rendered_path = render_path(operation, path_params)?;
        let method = parse_method(operation)?;
        self.inner
            .request_json_with_query(method, &rendered_path, query, body)
    }
}

/// Returns the default server URL from the `OpenAPI` spec.
///
/// This is the first element of the `OpenAPI` `servers` array when present.
pub fn openapi_default_server_url() -> &'static str {
    OPENAPI_DEFAULT_SERVER_URL
}

fn find_operation(operation_id: &str) -> Result<&'static OperationDefinition, ClientError> {
    OPENAPI_OPERATIONS
        .iter()
        .find(|op| op.operation_id == operation_id)
        .ok_or_else(|| ClientError::UnknownOperation(operation_id.to_owned()))
}

fn parse_method(operation: &OperationDefinition) -> Result<Method, ClientError> {
    Method::from_bytes(operation.method.as_bytes())
        .map_err(|_| ClientError::UnknownOperation(operation.operation_id.to_owned()))
}

fn render_path(
    operation: &OperationDefinition,
    path_params: &[(&str, &str)],
) -> Result<String, ClientError> {
    let mut rendered = operation.path_template.to_owned();

    for required_param in operation.path_params {
        let value = path_params
            .iter()
            .find(|(name, _)| name == required_param)
            .map(|(_, value)| *value)
            .ok_or_else(|| ClientError::MissingPathParameter {
                operation_id: operation.operation_id.to_owned(),
                parameter: (*required_param).to_owned(),
            })?;

        let placeholder = format!("{{{required_param}}}");
        rendered = rendered.replace(&placeholder, &encode_path_segment(value));
    }

    Ok(rendered)
}

fn encode_path_segment(value: &str) -> String {
    byte_serialize(value.as_bytes()).collect()
}

#[cfg(test)]
mod tests {
    use super::{IriClient, find_operation, render_path};
    use crate::ClientError;

    #[test]
    fn operation_catalog_is_non_empty() {
        assert!(!IriClient::operations().is_empty());
    }

    #[test]
    fn render_path_replaces_required_path_params() {
        let op = find_operation("getSite").expect("operation exists");
        let path = render_path(op, &[("site_id", "site-1")]).expect("path renders");
        assert_eq!(path, "/api/v1/facility/sites/site-1");
    }

    #[test]
    fn render_path_reports_missing_parameter() {
        let op = find_operation("getSite").expect("operation exists");
        let error = render_path(op, &[]).expect_err("missing parameter should error");
        match error {
            ClientError::MissingPathParameter {
                operation_id,
                parameter,
            } => {
                assert_eq!(operation_id, "getSite");
                assert_eq!(parameter, "site_id");
            }
            other => panic!("unexpected error: {other}"),
        }
    }
}
