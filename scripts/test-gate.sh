#!/usr/bin/env bash

set -euo pipefail

gate_timeout="${WARD_TEST_GATE_TIMEOUT:-8m}"

if ! command -v pre-commit >/dev/null 2>&1; then
  echo "ward: pre-commit not found on PATH" >&2
  exit 127
fi

echo "ward: pre-commit migration baseline run --all-files"
if command -v timeout >/dev/null 2>&1; then
  timeout --kill-after=30s "${gate_timeout}" pre-commit run --all-files
elif command -v gtimeout >/dev/null 2>&1; then
  gtimeout --kill-after=30s "${gate_timeout}" pre-commit run --all-files
else
  echo "ward: timeout unavailable; running pre-commit without an outer deadline"
  pre-commit run --all-files
fi

echo "ward: cargo test --workspace"
cargo test --workspace
