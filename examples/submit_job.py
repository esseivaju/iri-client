"""Submit a job with `launchJob` and poll `getJob` until terminal status.

Before running:
1. Build/install the extension module:
   maturin develop --features python
2. Export:
   - IRI_ACCESS_TOKEN (required for compute endpoints)
3. Optionally export:
   - IRI_RESOURCE_ID (default: perlmutter compute nodes)
   - IRI_BASE_URL (defaults to OpenAPI server URL)
   - IRI_POLL_INTERVAL_SECONDS (default: 5)
   - IRI_MAX_POLLS (default: 60)
"""

from __future__ import annotations

import json
import os
import time

from iri_client import Client

JOB_DIR='/pscratch/sd/e/<myuser>/iri-workdir'
EXEC='/pscratch/sd/e/<myuser>/iri-workdir/dummy_job.sh'
PROJECT='<mXXXX>'

# directory, stdout_path, and stderr_path are not working and defaulting to home directory

TERMINAL_STATES = {"completed", "failed", "canceled"}

JOB_SPEC: dict[str, object] = {
    "executable": f"{EXEC}",
    "arguments": [],
    "directory": f"{JOB_DIR}",
    "name": "my-job",
    "inherit_environment": True,
    "environment": {},
    "stdout_path": f"{JOB_DIR}/output.txt",
    "stderr_path": f"{JOB_DIR}/error.txt",
    "resources": {
        "node_count": 1,
        "process_count": 32,
        "processes_per_node": 32,
        "cpu_cores_per_process": 2,
        "gpu_cores_per_process": 1,
        "exclusive_node_use": True,
    },
    "attributes": {
        "duration": 30,
        "queue_name": "debug",
        "account": f"{PROJECT}",
        "custom_attributes": {},
    },
    "post_launch": "echo done",
    "launcher": "srun",
}


def call_operation_json(
    client: Client,
    operation_id: str,
    *,
    path_params: dict[str, object] | None = None,
    query: dict[str, object] | None = None,
    body: dict[str, object] | None = None,
) -> dict[str, object]:
    payload = client.call_operation(
        operation_id,
        path_params_json=json.dumps(path_params) if path_params else None,
        query_json=json.dumps(query) if query else None,
        body_json=json.dumps(body) if body else None,
    )
    return json.loads(payload)


def main() -> int:
    access_token = os.getenv("IRI_ACCESS_TOKEN")
    if not access_token:
        raise SystemExit("IRI_ACCESS_TOKEN is required.")

    resource_id = os.getenv("IRI_RESOURCE_ID", "b3af92a7-cf5f-42cf-a4be-6f6554a779e3")
    base_url = os.getenv("IRI_BASE_URL")
    poll_interval = int(os.getenv("IRI_POLL_INTERVAL_SECONDS", "5"))
    max_polls = int(os.getenv("IRI_MAX_POLLS", "60"))

    client = Client(base_url=base_url, access_token=access_token)

    created_job = call_operation_json(
        client,
        "launchJob",
        path_params={"resource_id": resource_id},
        body=JOB_SPEC,
    )
    job_id = created_job.get("id")
    if not isinstance(job_id, str) or not job_id:
        raise RuntimeError(f"launchJob did not return a valid job id: {created_job}")

    print(f"Submitted job id: {job_id}")
    print(json.dumps(created_job, indent=2, sort_keys=True))

    for attempt in range(1, max_polls + 1):
        try:
            job = call_operation_json(
                client,
                "getJob",
                path_params={"resource_id": resource_id, "job_id": job_id},
            )
        except RuntimeError as e:
            print(f"Request failed: {e}")
            continue
        status = job.get("status")
        if isinstance(status, dict):
            state = status.get("state", "unknown").strip().lower()
        else:
            state = str()
        print(f"[{attempt}/{max_polls}] state={state}")
        print(json.dumps(job, indent=2, sort_keys=True))

        if state in TERMINAL_STATES:
            return 0 if state == "completed" else 2

        if attempt < max_polls:
            time.sleep(poll_interval)

    print(
        "Job did not reach a terminal state "
        f"after {max_polls} polls ({poll_interval}s interval)."
    )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
