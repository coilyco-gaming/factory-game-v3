# Features

What `factory-game-v3` currently ships.

## Inventory

- **Completed Rust migration** - every retained Unity gameplay feature has a tested Rust implementation or an explicit obsolete decision. The C# gameplay tree, test project, and Unity-only dependency plugins are removed.
- **Retained art and asset surface** - `Assets/Scenes/`, `Assets/Materials/`, `Assets/Resources/Art/`, `Assets/TextMesh Pro/`, textures, shaders, and their metadata remain as inert visual inputs for future Bevy presentation work.
- **Rust simulation workspace** - `Cargo.toml` now defines a pure Rust workspace with `factory_content`, `factory_sim`, and `factory_cli` for closed-loop migration slices. The workspace stays Bevy-free and models the complete active Unity item catalog with typed content IDs, resource containers, mining extraction over finite deposits and manifest items, indexed multi-factory production, eight-neighbor automated inserters and retrievers, multi-ingredient recipes with configurable per-factory demand, typed collect/deliver/retrieve/deploy dispatch with nearest matching targets, deterministic multi-hauler arbitration, and factory-output supply, queued drill and general build-site deployment, spawned-structure occupancy, movement, ordered ore and drill teardown, scenario-owned occupied grids with deterministic A-star routing and transit-cell collision arbitration, node-owned batteries with capacity-weighted adjacent balancing, fuel-burning generation, automatic battery-backed power-line routing, energy-gated systems, tick stepping, run-summary metrics, and a headless runner. See [docs/factory-sim.md](factory-sim.md), [unity-parity.md](unity-parity.md), and [unity-feature-audit.md](unity-feature-audit.md).
- **Iron bars slice** - the starter scenario runs `IronOre -> IronBars` through a deterministic loop that mines a finite deposit at the source, hauls over the road, and crafts at the factory, with JSONL snapshots that expose mining, dispatch, topology, and movement state for debugging.
- **Bevy/Wasm simulation viewer** - `crates/factory_shell` hosts the deterministic simulation behind a native and browser-capable Bevy projection. It renders topology nodes, blocked cells, generated power lines, smoothly moving haulers including grid-transit positions, inventories, dispatch and deployment state, spawned structures, per-factory craft progress, aggregate power generation, node battery charge, focused-cell status, global object and inventory totals, run metrics, and a bounded rolling activity feed. Pulsing nodes, directional route dashes, cargo-aware badges, in-node craft and power gauges, and restrained industrial output chips animate material and energy flow between deterministic ticks. The continuous showcase cycles through all ten starter scenarios, including an adjacent five-factory mining-drill chain, a separated freight line, an automatic grid link, and warehouse construction. A visible cursor, bounded WASD or arrow-key camera movement, one-through-ten keyboard and wheel zoom, clickable control deck, and matching keyboard actions provide the retained player and playback controls. Bevy consumes immutable snapshots and owns no simulation rules. See [factory-viewer.md](factory-viewer.md) and [factory-shell.md](factory-shell.md).
- **Source-owned private web image publication** - Forgejo Actions uses the trusted deploy lane to build the Trunk/Wasm Dockerfile and publish `forgejo.coilysiren.me/coilyco-gaming/factory-game-v3:<full-source-sha>`. The deploy repo owns the read-only pull credential, rollout, and public exposure.
- **Current validation surface** - `ward exec test` runs the repo's pre-commit baseline plus the Rust workspace tests, and CI calls `bash scripts/test-gate.sh` directly for the same check.
- **C# decommission record** - the file-to-Rust proof and preserved asset boundary live in [csharp-decommission.md](csharp-decommission.md).

## See also

- [README.md](../README.md)
- [AGENTS.md](../AGENTS.md)
- [`.ward/ward.yaml`](../.ward/ward.yaml)
- [docs/features-release-tooling.md](features-release-tooling.md)
