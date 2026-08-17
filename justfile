# Per-repo task manifest. Run `just` (or `just --list`) to see every verb.
#
# Recipes take trailing arguments directly: `just <verb> a b`, where the
# retired form was `ward exec <verb> -- a b`.
#
# One line of comment per recipe on purpose: just reads only the LAST comment
# line above a recipe, so a wrapped description silently truncates to its tail.
#
# `ward exec` is retired. `.ward/ward.yaml` survives carrying catalog metadata
# only, because the catalog hooks upstream in agentic-os pin that exact path.

set positional-arguments

# Default target: list every available recipe.
default:
    @just --list --unsorted

# Run the pre-commit baseline and Rust workspace tests.
test *ARGS:
    @bash scripts/test-gate.sh "$@"

# Run the Rust workspace tests.
cargo-test *ARGS:
    @cargo test --workspace "$@"

# Format the Rust workspace with the repository's two-space style.
cargo-fmt *ARGS:
    @cargo fmt --all -- --config tab_spaces=2 "$@"

# Run the headless Rust scenario runner.
cargo-run *ARGS:
    @cargo run -p factory_cli -- "$@"

# Play the compact loop headlessly. One JSON request per line in, one response per line out.
play *ARGS:
    @cargo run -p factory_cli -- play "$@"

# Run the deterministic 650-tick v2 sustained-operation proof.
v2-liveness *ARGS:
    @cargo test --release -p factory_sim v2_world_sustains_remote_power_after_starter_charge "$@"

# Run the native Bevy simulation viewer with optimized graphics dependencies.
shell-run *ARGS:
    @cargo run --profile viewer -p factory_shell "$@"

# Build the Wasm/static viewer bundle via trunk.
shell-build-web *ARGS:
    @bash scripts/shell-web.sh build "$@"

# Build the deployable factory-game-v3 web image.
image-build *ARGS:
    @docker build -t factory-game-v3:local . "$@"

# Lint the trusted Forgejo OCI publisher shell contract.
check-publish *ARGS:
    @pre-commit run shellcheck --files scripts/publish-image.sh "$@"

# Serve the simulation viewer with trunk hot reload.
shell-serve *ARGS:
    @bash scripts/shell-web.sh serve "$@"
