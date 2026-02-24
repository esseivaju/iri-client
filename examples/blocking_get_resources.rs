//! Call a public operation with query parameters.
//!
//! Run:
//! `cargo run --example blocking_get_resources`
//!
//! Optional env vars:
//! - `IRI_BASE_URL` (defaults to OpenAPI server URL)
//! - `IRI_RESOURCE_LIMIT` (defaults to `5`)

use iri_client::BlockingIriClient;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = std::env::var("IRI_BASE_URL").ok();
    let limit = std::env::var("IRI_RESOURCE_LIMIT").unwrap_or_else(|_| "5".to_owned());

    let client = match base_url {
        Some(url) => BlockingIriClient::new(url)?,
        None => BlockingIriClient::from_openapi_default_server()?,
    };

    let resources = client.call_operation(
        "getResources",
        &[],
        &[("limit", limit.as_str()), ("offset", "0")],
        None,
    )?;

    println!("{}", serde_json::to_string_pretty(&resources)?);
    Ok(())
}
