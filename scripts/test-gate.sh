#!/usr/bin/env bash

set -euo pipefail

gate_timeout="${WARD_TEST_GATE_TIMEOUT:-8m}"

if ! command -v pre-commit >/dev/null 2>&1; then
  echo "ward: pre-commit not found on PATH" >&2
  exit 127
fi

echo "ward: pre-commit run --all-files"
timeout --kill-after=30s "${gate_timeout}" pre-commit run --all-files
