use reqwest::{Method, Url};
use serde_json::Value;

use crate::ClientError;

/// Generic blocking JSON REST client.
///
/// This is the synchronous counterpart of [`crate::ApiClient`].
#[derive(Debug)]
pub struct BlockingApiClient {
    base_url: Url,
    authorization_token: Option<String>,
    http: reqwest::blocking::Client,
}

impl BlockingApiClient {
    /// Creates a new client with the given base URL.
    ///
    /// The URL is normalized to include a trailing slash, so relative endpoint
    /// paths join correctly.
    pub fn new(base_url: impl AsRef<str>) -> Result<Self, ClientError> {
        let parsed = Url::parse(base_url.as_ref())
            .map_err(|_| ClientError::InvalidBaseUrl(base_url.as_ref().to_owned()))?;

        Ok(Self {
            base_url: ensure_trailing_slash(parsed),
            authorization_token: None,
            http: reqwest::blocking::Client::new(),
        })
    }

    /// Returns a new client with a raw access token attached to all requests.
    ///
    /// This sets `Authorization: <token>` (without `Bearer ` prefix).
    #[must_use]
    pub fn with_authorization_token(mut self, token: impl Into<String>) -> Self {
        self.authorization_token = Some(token.into());
        self
    }

    /// Sends a `GET` request and parses the response as JSON.
    pub fn get_json(&self, path: &str) -> Result<Value, ClientError> {
        self.request_json(Method::GET, path, None)
    }

    /// Sends a `GET` request with query parameters and parses the response as JSON.
    pub fn get_json_with_query(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<Value, ClientError> {
        self.request_json_with_query(Method::GET, path, query, None)
    }

    /// Sends a request and parses the response as JSON.
    ///
    /// Use [`Self::request_json_with_query`] when query parameters are needed.
    pub fn request_json(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
    ) -> Result<Value, ClientError> {
        self.request_json_with_query(method, path, &[], body)
    }

    /// Sends a request with query parameters and parses the response as JSON.
    ///
    /// Returns [`Value::Null`] for successful responses with an empty body.
    pub fn request_json_with_query(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, &str)],
        body: Option<Value>,
    ) -> Result<Value, ClientError> {
        let url = self.build_url(path)?;
        let mut request = self
            .http
            .request(method, url)
            .header(reqwest::header::ACCEPT, "application/json");

        if !query.is_empty() {
            request = request.query(query);
        }

        if let Some(token) = &self.authorization_token {
            request = request.bearer_auth(token);
        }

        if let Some(json_body) = body {
            request = request.json(&json_body);
        }

        let response = request.send()?;
        let status = response.status();
        let payload = response.text()?;

        if !status.is_success() {
            return Err(ClientError::HttpStatus {
                status,
                body: payload,
            });
        }

        if payload.trim().is_empty() {
            Ok(Value::Null)
        } else {
            Ok(serde_json::from_str(&payload)?)
        }
    }

    fn build_url(&self, path: &str) -> Result<Url, ClientError> {
        let relative = path.trim_start_matches('/');
        self.base_url
            .join(relative)
            .map_err(|_| ClientError::InvalidPath(path.to_owned()))
    }
}

fn ensure_trailing_slash(mut url: Url) -> Url {
    if !url.path().ends_with('/') {
        let mut path = url.path().to_owned();
        path.push('/');
        url.set_path(&path);
    }
    url
}
