//! Call an authenticated operation (`getProjects`) using the async `IriClient`.
//!
//! Run:
//! `IRI_ACCESS_TOKEN=<token> cargo run --example async_get_projects`
//!
//! Optional env vars:
//! - `IRI_BASE_URL` (defaults to OpenAPI server URL)

use iri_client::IriClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = match std::env::var("IRI_ACCESS_TOKEN") {
        Ok(value) => value,
        Err(_) => {
            eprintln!("Set IRI_ACCESS_TOKEN before running this example.");
            std::process::exit(2);
        }
    };

    let base_url = std::env::var("IRI_BASE_URL").ok();
    let client = match base_url {
        Some(url) => IriClient::new(url)?,
        None => IriClient::from_openapi_default_server()?,
    }
    .with_authorization_token(token);

    let projects = client.call_operation("getProjects", &[], &[], None).await?;
    println!("{}", serde_json::to_string_pretty(&projects)?);
    Ok(())
}
