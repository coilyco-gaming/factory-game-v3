# Features

What `factory-game-v3` currently ships. This repo is still in migration, so this inventory describes the current Unity/C# reference surface plus the first Rust migration slice.

## Inventory

- **Unity/C# migration reference state** - Unity project settings and editor/package scaffolding are gone, but the repo still holds the Unity-era gameplay reference surface while the Rust/Bevy rewrite is staged.
- **Retained art and asset surface** - `Assets/Scenes/`, `Assets/Materials/`, `Assets/Plugins/`, `Assets/TextMesh Pro/`, and the remaining `.meta` files stay in place as migration references.
- **Rust simulation workspace** - `Cargo.toml` now defines a pure Rust workspace with `factory_content`, `factory_sim`, and `factory_cli` for the first closed-loop migration slice. The workspace stays Bevy-free and models typed content IDs, resource containers, mining extraction over finite deposits and manifest items, multi-ingredient recipes with per-item factory demand, typed dispatch intents and assignments with deterministic multi-hauler arbitration, a hub-shaped world topology with N sources and N haulers, tick stepping, run-summary metrics, and a headless runner. See [docs/factory-sim.md](factory-sim.md).
- **Iron bars slice** - the starter scenario runs `IronOre -> IronBars` through a deterministic loop that mines a finite deposit at the source, hauls over the road, and crafts at the factory, with JSONL snapshots that expose mining, dispatch, topology, and movement state for debugging.
- **Bevy/Wasm simulation viewer** - `crates/factory_shell` hosts the deterministic simulation behind a native and browser-capable Bevy projection. It renders topology nodes, smoothly moving haulers, inventories, dispatch state, craft progress, run metrics, and a bounded rolling activity feed. Pulsing nodes, directional route dashes, cargo-aware badges, an in-node craft gauge, and restrained industrial output chips animate material flow between deterministic ticks. The continuous showcase cycles through all three starter scenarios. A clickable control deck and matching keyboard actions provide play, pause, step, reset, speed, direct scenario selection, and cycle locking. Bevy consumes immutable snapshots and owns no simulation rules. See [factory-viewer.md](factory-viewer.md) and [factory-shell.md](factory-shell.md).
- **Source-owned web image publication** - Forgejo Actions builds the Trunk/Wasm Dockerfile and publishes `factory-game-v3:<git-sha>` to the in-cluster registry. The deploy repo owns rollout and public exposure.
- **Current validation surface** - `ward exec test` runs the repo's pre-commit baseline plus the Rust workspace tests, and CI calls `bash scripts/test-gate.sh` directly for the same check. `tests.csproj` remains retained C# reference material only, not an active validator.

## See also

- [README.md](../README.md)
- [AGENTS.md](../AGENTS.md)
- [`.ward/ward.yaml`](../.ward/ward.yaml)
- [docs/features-release-tooling.md](features-release-tooling.md)
