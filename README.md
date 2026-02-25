# iri-client

Rust client + Python bindings for the [NERSC IRI API](https://api.iri.nersc.gov) OpenAPI 3.1
spec in `openapi/openapi.json`.

## Python Bindings

Install from PyPI:

```bash
pip install iri-client
```

or build the extension module locally:

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
operations = Client.operations()
print(f"Loaded {len(operations)} operations from generated catalog")
print("First 10 operations:")
for operation in operations[:10]:
    print(f"  - {operation.operation_id} ({operation.method} {operation.path_template})")

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

## Authentication Model

This API uses header `Authorization` with the raw access token value:

```http
Authorization: <access_token>
```

This client follows that behavior when you call `with_authorization_token(...)`
or pass `access_token=...` to the client's constructor in Python.

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

## CLI Tool

A small CLI binary `iri-cli` is included. One can install it through `cargo`. It is
currently not distributed in the python package.

```bash
cargo install iri-client --features cli
```

Here are a few examples on how to use the cli client.

```bash
# Show generated operations
iri-cli operations

# Filter operation ids
iri-cli operations --filter Job

# Call by OpenAPI operation id with query params
iri-cli call getResources --query group=perlmutter --query resource_type=compute

# Call operation with path params
iri-cli call getSite --path-param site_id=<site-id>

# Raw method/path request
iri-cli request GET /api/v1/facility/sites --query limit=5

# Authenticated operation
IRI_ACCESS_TOKEN=<access-token> \
  iri-cli call getProjects
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
