"""Example usage of the generated async `iri_client` Python bindings.

Before running:
1. Build/install the extension in your environment:
   maturin develop --features python
2. Optionally export:
   - IRI_ACCESS_TOKEN
   - IRI_BASE_URL (defaults to OpenAPI server from the Rust crate)
"""

from __future__ import annotations

import asyncio
import json
import os

from iri_client import AsyncClient


def print_pretty(label: str, payload_json: str) -> None:
    """Render returned JSON strings in a readable format."""
    try:
        parsed = json.loads(payload_json)
    except json.JSONDecodeError:
        print(f"{label} (raw): {payload_json}")
        return

    print(f"{label}:")
    print(json.dumps(parsed, indent=2, sort_keys=True))


async def main() -> int:
    access_token = os.getenv("IRI_ACCESS_TOKEN")
    base_url = os.getenv("IRI_BASE_URL")

    client = AsyncClient(base_url=base_url, access_token=access_token)

    operations = AsyncClient.operations()
    print(f"Loaded {len(operations)} operations from generated catalog")
    print("First 5 operations:")
    for operation in operations[:5]:
        print(f"  - {operation.operation_id} ({operation.method} {operation.path_template})")

    facility_json = await client.call_operation("getFacility")
    print_pretty("Facility (getFacility)", facility_json)

    if access_token:
        projects_json = await client.call_operation("getProjects")
        print_pretty("Projects (getProjects)", projects_json)
    else:
        print("Set IRI_ACCESS_TOKEN to run the getProjects auth example.")

    return 0


if __name__ == "__main__":
    raise SystemExit(asyncio.run(main()))
