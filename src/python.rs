use std::str::FromStr;

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;
use reqwest::Method;
use serde_json::Value;

use crate::{BlockingIriClient, IriClient};

/// Metadata for one generated `OpenAPI` operation.
///
/// Returned by `Client.operations()` and `AsyncClient.operations()`.
#[pyclass(name = "OperationDefinition", get_all)]
pub struct PyOperationDefinition {
    /// Stable `OpenAPI` operation identifier (for example, `"getFacility"`).
    pub operation_id: String,
    /// Uppercase HTTP method (for example, `"GET"` or `"POST"`).
    pub method: String,
    /// Path template which may contain `{param}` placeholders.
    pub path_template: String,
    /// Required path-parameter names extracted from `path_template`.
    pub path_params: Vec<String>,
}

/// Synchronous Python client for the IRI API.
///
/// This class wraps the blocking Rust client and returns JSON payloads as
/// strings.
#[pyclass(name = "Client")]
pub struct PyClient {
    inner: BlockingIriClient,
}

/// Asynchronous Python client for the IRI API.
///
/// Methods return awaitables using the Tokio runtime integration from
/// `pyo3-async-runtimes`.
#[pyclass(name = "AsyncClient")]
pub struct PyAsyncClient {
    inner: IriClient,
}

#[pymethods]
impl PyClient {
    /// Create a new synchronous client.
    ///
    /// Args:
    ///     `base_url`: Base API URL. If omitted, uses the default server from the `OpenAPI` spec.
    ///     `access_token`: Optional raw token sent as `Authorization: <token>`.
    #[new]
    #[pyo3(signature = (base_url=None, access_token=None))]
    fn new(base_url: Option<String>, access_token: Option<String>) -> PyResult<Self> {
        let client = match base_url {
            Some(url) => BlockingIriClient::new(url).map_err(to_py_value_error)?,
            None => BlockingIriClient::from_openapi_default_server().map_err(to_py_value_error)?,
        };
        let client = if let Some(value) = access_token {
            client.with_authorization_token(value)
        } else {
            client
        };

        Ok(Self { inner: client })
    }

    /// Return all generated `OpenAPI` operation definitions.
    #[staticmethod]
    fn operations() -> Vec<PyOperationDefinition> {
        operations_for_python()
    }

    /// Perform a `GET` request against a raw API path.
    fn get(&self, path: &str) -> PyResult<String> {
        self.request("GET", path, None, None)
    }

    /// Perform a raw HTTP request by method and path.
    ///
    /// Args:
    ///     method: HTTP method (for example, `"GET"`).
    ///     path: API path relative to the configured base URL.
    ///     `query_json`: Optional JSON object (string) used as query parameters.
    ///     `body_json`: Optional JSON value (string) used as request body.
    ///
    /// Returns:
    ///     A JSON string containing the parsed response payload.
    #[pyo3(signature = (method, path, query_json=None, body_json=None))]
    fn request(
        &self,
        method: &str,
        path: &str,
        query_json: Option<String>,
        body_json: Option<String>,
    ) -> PyResult<String> {
        let parsed_method = Method::from_str(method)
            .map_err(|e| PyValueError::new_err(format!("invalid HTTP method: {e}")))?;
        let query_pairs = parse_map_arg(query_json)?;
        let borrowed_query: Vec<(&str, &str)> = query_pairs
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
            .collect();
        let body: Option<Value> = body_json
            .map(|raw| serde_json::from_str(&raw).map_err(to_py_value_error))
            .transpose()?;

        let value = self
            .inner
            .request_json_with_query(parsed_method, path, &borrowed_query, body)
            .map_err(to_py_runtime_error)?;

        Ok(value.to_string())
    }

    /// Call an endpoint by `OpenAPI` `operation_id`.
    ///
    /// Args:
    ///     `operation_id`: Operation identifier from `Client.operations()`.
    ///     `path_params_json`: Optional JSON object (string) for path parameters.
    ///     `query_json`: Optional JSON object (string) for query parameters.
    ///     `body_json`: Optional JSON value (string) for request body.
    ///
    /// Returns:
    ///     A JSON string containing the parsed response payload.
    #[pyo3(signature = (operation_id, path_params_json=None, query_json=None, body_json=None))]
    fn call_operation(
        &self,
        operation_id: &str,
        path_params_json: Option<String>,
        query_json: Option<String>,
        body_json: Option<String>,
    ) -> PyResult<String> {
        let path_pairs = parse_map_arg(path_params_json)?;
        let borrowed_path: Vec<(&str, &str)> = path_pairs
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
            .collect();
        let query_pairs = parse_map_arg(query_json)?;
        let borrowed_query: Vec<(&str, &str)> = query_pairs
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
            .collect();
        let body: Option<Value> = body_json
            .map(|raw| serde_json::from_str(&raw).map_err(to_py_value_error))
            .transpose()?;

        let value = self
            .inner
            .call_operation(operation_id, &borrowed_path, &borrowed_query, body)
            .map_err(to_py_runtime_error)?;

        Ok(value.to_string())
    }
}

#[pymethods]
impl PyAsyncClient {
    /// Create a new asynchronous client.
    ///
    /// Args:
    ///     `base_url`: Base API URL. If omitted, uses the default server from the `OpenAPI` spec.
    ///     `access_token`: Optional raw token sent as `Authorization: <token>`.
    #[new]
    #[pyo3(signature = (base_url=None, access_token=None))]
    fn new(base_url: Option<String>, access_token: Option<String>) -> PyResult<Self> {
        let client = match base_url {
            Some(url) => IriClient::new(url).map_err(to_py_value_error)?,
            None => IriClient::from_openapi_default_server().map_err(to_py_value_error)?,
        };
        let client = if let Some(value) = access_token {
            client.with_authorization_token(value)
        } else {
            client
        };

        Ok(Self { inner: client })
    }

    /// Return all generated `OpenAPI` operation definitions.
    #[staticmethod]
    fn operations() -> Vec<PyOperationDefinition> {
        operations_for_python()
    }

    /// Perform an asynchronous `GET` request against a raw API path.
    fn get<'py>(&self, py: Python<'py>, path: String) -> PyResult<Bound<'py, PyAny>> {
        self.request(py, "GET", path, None, None)
    }

    /// Perform an asynchronous raw HTTP request by method and path.
    ///
    /// Args:
    ///     method: HTTP method (for example, `"GET"`).
    ///     path: API path relative to the configured base URL.
    ///     `query_json`: Optional JSON object (string) used as query parameters.
    ///     `body_json`: Optional JSON value (string) used as request body.
    ///
    /// Returns:
    ///     An awaitable resolving to a JSON string response payload.
    #[pyo3(signature = (method, path, query_json=None, body_json=None))]
    fn request<'py>(
        &self,
        py: Python<'py>,
        method: &str,
        path: String,
        query_json: Option<String>,
        body_json: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let parsed_method = Method::from_str(method)
            .map_err(|e| PyValueError::new_err(format!("invalid HTTP method: {e}")))?;
        let query_pairs = parse_map_arg(query_json)?;
        let body = parse_body_arg(body_json)?;
        let client = self.inner.clone();

        future_into_py(py, async move {
            let borrowed_query: Vec<(&str, &str)> = query_pairs
                .iter()
                .map(|(key, value)| (key.as_str(), value.as_str()))
                .collect();
            let value = client
                .request_json_with_query(parsed_method, &path, &borrowed_query, body)
                .await
                .map_err(to_py_runtime_error)?;
            Ok(value.to_string())
        })
    }

    /// Call an endpoint asynchronously by `OpenAPI` `operation_id`.
    ///
    /// Args:
    ///     `operation_id`: Operation identifier from `AsyncClient.operations()`.
    ///     `path_params_json`: Optional JSON object (string) for path parameters.
    ///     `query_json`: Optional JSON object (string) for query parameters.
    ///     `body_json`: Optional JSON value (string) for request body.
    ///
    /// Returns:
    ///     An awaitable resolving to a JSON string response payload.
    #[pyo3(signature = (operation_id, path_params_json=None, query_json=None, body_json=None))]
    fn call_operation<'py>(
        &self,
        py: Python<'py>,
        operation_id: String,
        path_params_json: Option<String>,
        query_json: Option<String>,
        body_json: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let path_pairs = parse_map_arg(path_params_json)?;
        let query_pairs = parse_map_arg(query_json)?;
        let body = parse_body_arg(body_json)?;
        let client = self.inner.clone();

        future_into_py(py, async move {
            let borrowed_path: Vec<(&str, &str)> = path_pairs
                .iter()
                .map(|(key, value)| (key.as_str(), value.as_str()))
                .collect();
            let borrowed_query: Vec<(&str, &str)> = query_pairs
                .iter()
                .map(|(key, value)| (key.as_str(), value.as_str()))
                .collect();
            let value = client
                .call_operation(&operation_id, &borrowed_path, &borrowed_query, body)
                .await
                .map_err(to_py_runtime_error)?;
            Ok(value.to_string())
        })
    }
}

/// Python module entrypoint for the `iri_client` extension.
#[pymodule(name = "_iri_client")]
fn iri_client(_py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyOperationDefinition>()?;
    module.add_class::<PyClient>()?;
    module.add_class::<PyAsyncClient>()?;
    Ok(())
}

fn to_py_value_error(error: impl std::fmt::Display) -> PyErr {
    PyValueError::new_err(error.to_string())
}

fn to_py_runtime_error(error: impl std::fmt::Display) -> PyErr {
    PyRuntimeError::new_err(error.to_string())
}

fn operations_for_python() -> Vec<PyOperationDefinition> {
    BlockingIriClient::operations()
        .iter()
        .map(|op| PyOperationDefinition {
            operation_id: op.operation_id.to_owned(),
            method: op.method.to_owned(),
            path_template: op.path_template.to_owned(),
            path_params: op
                .path_params
                .iter()
                .map(|value| (*value).to_owned())
                .collect(),
        })
        .collect()
}

fn parse_body_arg(raw_json: Option<String>) -> PyResult<Option<Value>> {
    raw_json
        .map(|raw| serde_json::from_str(&raw).map_err(to_py_value_error))
        .transpose()
}

fn parse_map_arg(raw_json: Option<String>) -> PyResult<Vec<(String, String)>> {
    let Some(raw_json) = raw_json else {
        return Ok(Vec::new());
    };

    let value: Value = serde_json::from_str(&raw_json).map_err(to_py_value_error)?;
    let object = value
        .as_object()
        .ok_or_else(|| PyValueError::new_err("expected a JSON object"))?;

    Ok(object
        .iter()
        .map(|(key, value)| {
            let rendered = match value.as_str() {
                Some(as_str) => as_str.to_owned(),
                None => value.to_string(),
            };
            (key.to_owned(), rendered)
        })
        .collect())
}
