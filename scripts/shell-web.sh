#!/usr/bin/env bash
# Trunk must run from the crate directory: invoked from the workspace root it
# finds the virtual manifest and fails with "could not find the root package".
set -euo pipefail

cd "$(dirname "$0")/../crates/factory_shell"

# Wasm targets belong to rustup even when Homebrew Rust appears first.
rustup_bin="${HOME}/.cargo/bin"
if [[ -x "${rustup_bin}/cargo" ]]; then
  export PATH="${rustup_bin}:${PATH}"
fi

# Trunk parses NO_COLOR as a boolean instead of the standard presence flag.
if [[ -n "${NO_COLOR+x}" ]]; then
  export NO_COLOR=true
fi

case "${1:-build}" in
  build) trunk build ;;
  serve) trunk serve ;;
  *)
    echo "usage: shell-web.sh [build|serve]" >&2
    exit 2
    ;;
esac
