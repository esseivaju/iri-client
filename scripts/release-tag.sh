#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Create release tags for client and OpenAPI versions on HEAD.

Usage:
  scripts/release-tag.sh [options]

Options:
  --client-version <version>  Client version without leading "v" (overrides Cargo.toml).
  --cargo-manifest <path>     Cargo manifest path (default: Cargo.toml).
  --openapi-file <path>       OpenAPI spec file path (default: openapi/openapi.json).
  --allow-dirty               Allow tagging with a dirty working tree.
  --dry-run                   Show resolved tags and commands without creating tags.
  -h, --help                  Show this help.

Tag format:
  client tag: v<client-version>
  api tag:    api/v<openapi-info.version>
EOF
}

client_version=""
cargo_manifest="Cargo.toml"
openapi_file="openapi/openapi.json"
allow_dirty=false
dry_run=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --client-version)
      client_version="${2:-}"
      shift 2
      ;;
    --cargo-manifest)
      cargo_manifest="${2:-}"
      shift 2
      ;;
    --openapi-file)
      openapi_file="${2:-}"
      shift 2
      ;;
    --allow-dirty)
      allow_dirty=true
      shift
      ;;
    --dry-run)
      dry_run=true
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ ! -f "$cargo_manifest" ]]; then
  echo "error: cargo manifest not found: $cargo_manifest" >&2
  exit 1
fi

if [[ -z "$client_version" ]]; then
  client_version="$(
    python3 - "$cargo_manifest" <<'PY'
import sys

path = sys.argv[1]
in_package = False
with open(path, "r", encoding="utf-8") as fh:
    for line in fh:
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        if stripped.startswith("[") and stripped.endswith("]"):
            in_package = stripped == "[package]"
            continue
        if in_package and stripped.startswith("version"):
            key, _, raw_value = stripped.partition("=")
            if key.strip() == "version":
                value = raw_value.strip().strip('"').strip("'")
                if value:
                    print(value)
                    break
    else:
        raise SystemExit("error: package version not found in Cargo.toml")
PY
  )"
fi

if ! [[ "$client_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.-]+)?$ ]]; then
  echo "error: invalid client version format: $client_version" >&2
  echo "expected something like 1.2.3 or 1.2.3-rc.1" >&2
  exit 1
fi

if [[ ! -f "$openapi_file" ]]; then
  echo "error: openapi file not found: $openapi_file" >&2
  exit 1
fi

if ! git rev-parse --git-dir >/dev/null 2>&1; then
  echo "error: this command must run inside a git repository" >&2
  exit 1
fi

if ! $allow_dirty; then
  if [[ -n "$(git status --porcelain)" ]]; then
    echo "error: working tree is not clean; commit or stash changes first" >&2
    echo "use --allow-dirty to bypass this guard" >&2
    exit 1
  fi
fi

api_version="$(
  python3 - "$openapi_file" <<'PY'
import json
import sys

path = sys.argv[1]
with open(path, "r", encoding="utf-8") as fh:
    data = json.load(fh)

version = data.get("info", {}).get("version")
if not version or not isinstance(version, str):
    raise SystemExit("error: OpenAPI info.version is missing or invalid")

print(version.strip())
PY
)"

if [[ -z "$api_version" ]]; then
  echo "error: failed to extract OpenAPI info.version from $openapi_file" >&2
  exit 1
fi

client_tag="v${client_version}"
api_tag="api/v${api_version}"
head_sha="$(git rev-parse --verify HEAD)"

for tag in "$client_tag" "$api_tag"; do
  if git rev-parse -q --verify "refs/tags/${tag}" >/dev/null; then
    echo "error: tag already exists: ${tag}" >&2
    exit 1
  fi
done

client_msg="Client ${client_version}; OpenAPI ${api_version}"
api_msg="OpenAPI ${api_version}; client ${client_version}"

echo "HEAD:       ${head_sha}"
echo "client tag: ${client_tag}"
echo "api tag:    ${api_tag}"
echo "spec:       ${openapi_file}"

if $dry_run; then
  echo
  echo "Dry run. Commands:"
  echo "  git tag -a \"${client_tag}\" -m \"${client_msg}\" \"${head_sha}\""
  echo "  git tag -a \"${api_tag}\" -m \"${api_msg}\" \"${head_sha}\""
  echo "  git push origin \"${client_tag}\" \"${api_tag}\""
  exit 0
fi

git tag -a "${client_tag}" -m "${client_msg}" "${head_sha}"
git tag -a "${api_tag}" -m "${api_msg}" "${head_sha}"

echo
echo "Created tags on ${head_sha}:"
echo "  ${client_tag}"
echo "  ${api_tag}"
echo
echo "Push when ready:"
echo "  git push origin \"${client_tag}\" \"${api_tag}\""
