# iri-client

Rust client + Python bindings for the NERSC IRI API, driven by the OpenAPI 3.1
spec in `openapi/openapi.json`.

## Authentication Model

This API uses header `Authorization` with the raw access token value:

```http
Authorization: <access_token>
```

This client follows that behavior when you call `with_authorization_token(...)`
or pass `access_token=...` in Python.

## Runnable Cargo Examples

These are checked-in Rust examples you can run directly:

```bash
# Lists generated operation ids/methods/paths (no network calls)
cargo run --example blocking_list_operations

# Calls getResources with query parameters
cargo run --example blocking_get_resources

# Same example with custom base URL and limit
IRI_BASE_URL=https://api.iri.nersc.gov IRI_RESOURCE_LIMIT=10 \
  cargo run --example blocking_get_resources

# Calls auth-required getProjects
IRI_ACCESS_TOKEN=<access-token> cargo run --example blocking_get_projects

# Async IriClient examples
cargo run --example async_get_resources
IRI_ACCESS_TOKEN=<access-token> cargo run --example async_get_projects

# Async ApiClient example (raw path + query)
cargo run --example async_api_client_sites
```

## CLI Tool (Async)

A small async CLI binary is included at `src/bin/iri-cli.rs`.

```bash
# Show generated operations
cargo run --features cli --bin iri-cli -- operations

# Filter operation ids
cargo run --features cli --bin iri-cli -- operations --filter Job

# Call by OpenAPI operation id with query params
cargo run --features cli --bin iri-cli -- call getResources --query limit=5 --query offset=0

# Call operation with path params
cargo run --features cli --bin iri-cli -- call getSite --path-param site_id=<site-id>

# Raw method/path request
cargo run --features cli --bin iri-cli -- request GET /api/v1/facility/sites --query limit=5

# Authenticated operation
IRI_ACCESS_TOKEN=<access-token> \
  cargo run --features cli --bin iri-cli -- call getProjects
```

## Rust Examples (OpenAPI Operation Client)

Use `IriClient` when calling by OpenAPI `operationId`.

### Public endpoint: `getFacility`

```rust
use iri_client::IriClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = IriClient::from_openapi_default_server()?;

    let facility = client
        .call_operation("getFacility", &[], &[], None)
        .await?;

    println!("{facility}");
    Ok(())
}
```

### Query parameters: `getResources`

```rust
use iri_client::IriClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = IriClient::new("https://api.iri.nersc.gov")?;

    let resources = client
        .call_operation(
            "getResources",
            &[],
            &[("limit", "10"), ("offset", "0"), ("resource_type", "compute")],
            None,
        )
        .await?;

    println!("{resources}");
    Ok(())
}
```

### Path parameter: `getSite`

```rust
use iri_client::IriClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = IriClient::new("https://api.iri.nersc.gov")?;

    let site = client
        .call_operation("getSite", &[("site_id", "dd7f822a-3ad2-54ae-bddb-796ee07bd206")], &[], None)
        .await?;

    println!("{site}");
    Ok(())
}
```

### Auth-required endpoint: `getProjects`

```rust
use iri_client::IriClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let access_token = std::env::var("IRI_ACCESS_TOKEN")?;
    let client = IriClient::new("https://api.iri.nersc.gov")?
        .with_authorization_token(access_token);

    let projects = client
        .call_operation("getProjects", &[], &[], None)
        .await?;

    println!("{projects}");
    Ok(())
}
```

## Rust Examples (Generic REST Client)

Use `ApiClient` when you want direct method/path control.

```rust
use iri_client::ApiClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ApiClient::new("https://api.iri.nersc.gov")?;

    let sites = client
        .get_json_with_query("/api/v1/facility/sites", &[("limit", "5"), ("offset", "0")])
        .await?;

    println!("{sites}");
    Ok(())
}
```

## Python Bindings

Build the extension module locally:

```bash
# Use maturin >= 1.9.4
maturin develop --features python
```

### Python operation examples

```python
import json
from iri_client import Client

client = Client(base_url="https://api.iri.nersc.gov")

# List operation ids
operation_ids = Client.operation_ids()
print(f"Loaded {len(operation_ids)} operations from generated catalog")
print("First 10 operation ids:")
for operation_id in operation_ids[:10]:
    print(f"  - {operation_id}")

# Public operation
print(client.call_operation("getFacility"))

# Path params
print(
    client.call_operation(
        "getSite",
        path_params_json=json.dumps({"site_id": "dd7f822a-3ad2-54ae-bddb-796ee07bd206"}),
    )
)

# Auth-required operation
# access_token = "<token from OAuth2>"
# auth_client = Client(base_url="https://api.iri.nersc.gov", access_token=access_token)
# print(auth_client.call_operation("getProjects"))
```

Full runnable Python operation script:
- `examples/python_module_example.py`

OAuth2 token helper script (`authlib` + `PrivateKeyJWT`):
- `examples/generate_auth_token.py`
