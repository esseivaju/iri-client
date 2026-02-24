//! Rust and Python-facing client library for the IRI REST API.
//!
//! Public API layers:
//! - [`ApiClient`]/[`BlockingApiClient`]: generic JSON HTTP clients.
//! - [`IriClient`]/[`BlockingIriClient`]: OpenAPI-driven operation clients.
//! - [`ClientError`]: unified error type used by all clients.
//!
//! The `OpenAPI` operation registry is generated at build time from
//! `openapi/openapi.json`.

mod blocking_client;
mod client;
mod error;
mod openapi_client;

/// Generic blocking JSON REST client.
pub use blocking_client::BlockingApiClient;
/// Generic async JSON REST client.
pub use client::ApiClient;
/// Error type returned by all client operations.
pub use error::ClientError;
/// OpenAPI-backed blocking operation client.
///
/// See also [`IriClient`] for the async variant.
pub use openapi_client::{
    BlockingIriClient, IriClient, OperationDefinition, openapi_default_server_url,
};

#[cfg(feature = "python")]
mod python;
