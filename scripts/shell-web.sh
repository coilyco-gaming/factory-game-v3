#!/usr/bin/env bash
# Trunk must run from the crate directory: invoked from the workspace root it
# finds the virtual manifest and fails with "could not find the root package".
set -euo pipefail

cd "$(dirname "$0")/../crates/factory_shell"

case "${1:-build}" in
  build) trunk build ;;
  serve) trunk serve ;;
  *)
    echo "usage: shell-web.sh [build|serve]" >&2
    exit 2
    ;;
esac
