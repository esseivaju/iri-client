//! Call a raw path with query parameters using the async `ApiClient`.
//!
//! Run:
//! `cargo run --example async_api_client_sites`
//!
//! Optional env vars:
//! - `IRI_BASE_URL` (defaults to OpenAPI server URL)
//! - `IRI_SITE_LIMIT` (defaults to `5`)

use iri_client::{ApiClient, openapi_default_server_url};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url =
        std::env::var("IRI_BASE_URL").unwrap_or_else(|_| openapi_default_server_url().to_owned());
    let limit = std::env::var("IRI_SITE_LIMIT").unwrap_or_else(|_| "5".to_owned());

    let client = ApiClient::new(base_url)?;
    let sites = client
        .get_json_with_query(
            "/api/v1/facility/sites",
            &[("limit", limit.as_str()), ("offset", "0")],
        )
        .await?;

    println!("{}", serde_json::to_string_pretty(&sites)?);
    Ok(())
}
