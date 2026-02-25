"""Example usage of the generated `iri_client` Python module.

Before running:
1. Build/install the extension in your environment:
   maturin develop --features python
2. Optionally export:
   - IRI_ACCESS_TOKEN
   - IRI_BASE_URL (defaults to OpenAPI server from the Rust crate)
   - IRI_RESOURCE_LIMIT (default: 5)
   - IRI_SITE_ID (for getSite example)
"""

from __future__ import annotations

import json
import os

from iri_client import Client


def print_pretty(label: str, payload_json: str) -> None:
    """Render returned JSON strings in a readable format."""
    try:
        parsed = json.loads(payload_json)
    except json.JSONDecodeError:
        print(f"{label} (raw): {payload_json}")
        return

    print(f"{label}:")
    print(json.dumps(parsed, indent=2, sort_keys=True))


def call_and_print(
    client: Client,
    label: str,
    operation_id: str,
    *,
    path_params: dict[str, str] | None = None,
    query: dict[str, object] | None = None,
    body: dict[str, object] | None = None,
) -> None:
    """Call one operation and print JSON, with light error handling."""
    path_params_json = json.dumps(path_params) if path_params else None
    query_json = json.dumps(query) if query else None
    body_json = json.dumps(body) if body else None

    try:
        payload_json = client.call_operation(
            operation_id,
            path_params_json=path_params_json,
            query_json=query_json,
            body_json=body_json,
        )
    except Exception as exc:  # pragma: no cover - runtime demo error path
        print(f"{label} failed: {exc}")
        return

    print_pretty(label, payload_json)


def main() -> int:
    access_token = os.getenv("IRI_ACCESS_TOKEN")
    base_url = os.getenv("IRI_BASE_URL")

    # If base_url is None, the client uses the OpenAPI default server URL.
    client = Client(base_url=base_url, access_token=access_token)

    operations = Client.operations()
    print(f"Loaded {len(operations)} operations from generated catalog")
    print("First 10 operations:")
    for operation in operations[:10]:
        print(
            f"  - {operation.operation_id} "
            f"({operation.method} {operation.path_template})"
        )

    # Operation with no path/query/body parameters.
    call_and_print(client, "Facility (getFacility)", "getFacility")

    # Operation with query parameters.
    resource_limit = int(os.getenv("IRI_RESOURCE_LIMIT", "5"))
    call_and_print(
        client,
        "Resources (getResources, query params)",
        "getResources",
        query={"limit": resource_limit, "offset": 0},
    )

    # Operation with path parameters.
    site_id = os.getenv("IRI_SITE_ID")
    if site_id:
        call_and_print(
            client,
            f"Site ({site_id})",
            "getSite",
            path_params={"site_id": site_id},
        )
    else:
        print("Set IRI_SITE_ID to run the getSite path-parameter example.")

    # Operation that typically requires access-token authentication.
    if access_token:
        call_and_print(
            client,
            "Projects (getProjects, access token required)",
            "getProjects",
        )
    else:
        print("Set IRI_ACCESS_TOKEN to run the getProjects auth example.")

    # Raw request path example (not operation-id based).
    health_json = client.request("GET", "/api/v1/facility")
    print_pretty("Raw request example", health_json)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
