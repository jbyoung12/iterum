#!/bin/sh
set -eu

if [ "$#" -lt 3 ]; then
  echo "usage: retrieve_context.sh <namespace> <tool_name> <resource_id> [error_text]" >&2
  exit 2
fi

NAMESPACE="$1"
TOOL_NAME="$2"
RESOURCE_ID="$3"
ERROR_TEXT="${4-}"
BASE_URL="${ITERUM_BASE_URL:-http://127.0.0.1:8000}"

python3 - "$NAMESPACE" "$TOOL_NAME" "$RESOURCE_ID" "$ERROR_TEXT" "$BASE_URL" <<'PY'
import json
import sys
import urllib.error
import urllib.request

namespace, tool_name, resource_id, error_text, base_url = sys.argv[1:6]
payload = {
    "namespace": namespace,
    "tool_name": tool_name,
    "resource_id": resource_id,
    "task_type": "analysis",
}
if error_text:
    payload["error_text"] = error_text

req = urllib.request.Request(
    f"{base_url.rstrip('/')}/v1/context/retrieve",
    data=json.dumps(payload).encode("utf-8"),
    headers={"content-type": "application/json"},
    method="POST",
)

try:
    with urllib.request.urlopen(req, timeout=5) as resp:
        sys.stdout.write(resp.read().decode("utf-8"))
except urllib.error.URLError as exc:
    sys.stderr.write(f"iterum unavailable: {exc}\n")
    sys.exit(1)
PY
