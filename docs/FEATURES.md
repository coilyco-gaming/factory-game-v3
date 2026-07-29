# Features

What `factory-game-v3` currently ships. This repo is still in migration, so this inventory describes the current Unity/C# reference surface plus the first Rust migration slice.

## Inventory

- **Unity/C# migration reference state** - Unity project settings and editor/package scaffolding are gone, but the repo still holds the Unity-era gameplay reference surface while the Rust/Bevy rewrite is staged.
- **Retained art and asset surface** - `Assets/Scenes/`, `Assets/Materials/`, `Assets/Plugins/`, `Assets/TextMesh Pro/`, and the remaining `.meta` files stay in place as migration references.
- **Rust simulation workspace** - `Cargo.toml` now defines a pure Rust workspace with `factory_content`, `factory_sim`, and `factory_cli` for closed-loop migration slices. The workspace stays Bevy-free and models typed content IDs, resource containers, mining extraction over finite deposits and manifest items, multi-ingredient recipes with per-item factory demand, typed collect/deliver/retrieve/deploy dispatch with deterministic multi-hauler arbitration, queued drill deployment and movement, scenario-owned occupied grids with deterministic A-star routing and transit-cell collision arbitration, a fuel-burning finite power grid with energy-gated systems, tick stepping, run-summary metrics, and a headless runner. See [docs/factory-sim.md](factory-sim.md) and [unity-parity.md](unity-parity.md).
- **Iron bars slice** - the starter scenario runs `IronOre -> IronBars` through a deterministic loop that mines a finite deposit at the source, hauls over the road, and crafts at the factory, with JSONL snapshots that expose mining, dispatch, topology, and movement state for debugging.
- **Bevy/Wasm simulation viewer** - `crates/factory_shell` hosts the deterministic simulation behind a native and browser-capable Bevy projection. It renders topology nodes and blocked cells, smoothly moving haulers including grid-transit positions, inventories, dispatch and deployment state, craft progress, power generation and charge, run metrics, and a bounded rolling activity feed. Pulsing nodes, directional route dashes, cargo-aware badges, in-node craft and power gauges, and restrained industrial output chips animate material and energy flow between deterministic ticks. The continuous showcase cycles through all six starter scenarios. A clickable control deck and matching keyboard actions provide play, pause, step, reset, speed, direct scenario selection, and cycle locking. Bevy consumes immutable snapshots and owns no simulation rules. See [factory-viewer.md](factory-viewer.md) and [factory-shell.md](factory-shell.md).
- **Source-owned private web image publication** - Forgejo Actions uses the trusted deploy lane to build the Trunk/Wasm Dockerfile and publish `forgejo.coilysiren.me/coilyco-gaming/factory-game-v3:<full-source-sha>`. The deploy repo owns the read-only pull credential, rollout, and public exposure.
- **Current validation surface** - `ward exec test` runs the repo's pre-commit baseline plus the Rust workspace tests, and CI calls `bash scripts/test-gate.sh` directly for the same check. `tests.csproj` remains retained C# reference material only, not an active validator.

## See also

- [README.md](../README.md)
- [AGENTS.md](../AGENTS.md)
- [`.ward/ward.yaml`](../.ward/ward.yaml)
- [docs/features-release-tooling.md](features-release-tooling.md)
