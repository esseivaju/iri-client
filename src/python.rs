use std::str::FromStr;
use std::sync::Mutex;

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use reqwest::Method;
use serde_json::Value;

use crate::BlockingIriClient;

#[pyclass(name = "OperationDefinition", get_all)]
pub struct PyOperationDefinition {
    pub operation_id: String,
    pub method: String,
    pub path_template: String,
    pub path_params: Vec<String>,
}

#[pyclass(name = "Client")]
pub struct PyClient {
    inner: Mutex<BlockingIriClient>,
}

#[pymethods]
impl PyClient {
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

        Ok(Self {
            inner: Mutex::new(client),
        })
    }

    #[staticmethod]
    fn operations() -> Vec<PyOperationDefinition> {
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

    fn get(&self, path: String) -> PyResult<String> {
        self.request("GET".to_owned(), path, None, None)
    }

    #[pyo3(signature = (method, path, query_json=None, body_json=None))]
    fn request(
        &self,
        method: String,
        path: String,
        query_json: Option<String>,
        body_json: Option<String>,
    ) -> PyResult<String> {
        let parsed_method = Method::from_str(&method)
            .map_err(|e| PyValueError::new_err(format!("invalid HTTP method: {e}")))?;
        let query_pairs = parse_map_arg(query_json)?;
        let borrowed_query: Vec<(&str, &str)> = query_pairs
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
            .collect();
        let body: Option<Value> = body_json
            .map(|raw| serde_json::from_str(&raw).map_err(to_py_value_error))
            .transpose()?;

        let client = self
            .inner
            .lock()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let value = client
            .request_json_with_query(parsed_method, &path, &borrowed_query, body)
            .map_err(to_py_runtime_error)?;

        Ok(value.to_string())
    }

    #[pyo3(signature = (operation_id, path_params_json=None, query_json=None, body_json=None))]
    fn call_operation(
        &self,
        operation_id: String,
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

        let client = self
            .inner
            .lock()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let value = client
            .call_operation(&operation_id, &borrowed_path, &borrowed_query, body)
            .map_err(to_py_runtime_error)?;

        Ok(value.to_string())
    }
}

#[pymodule]
fn iri_client(_py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyOperationDefinition>()?;
    module.add_class::<PyClient>()?;
    Ok(())
}

fn to_py_value_error(error: impl std::fmt::Display) -> PyErr {
    PyValueError::new_err(error.to_string())
}

fn to_py_runtime_error(error: impl std::fmt::Display) -> PyErr {
    PyRuntimeError::new_err(error.to_string())
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
