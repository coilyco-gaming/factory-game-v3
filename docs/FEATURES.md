# Features

What `factory-game-v3` currently ships.

## Inventory

- **Completed Rust gameplay migration** - the C# gameplay tree, test project, and Unity-only dependency plugins are removed. The source-level audit against the final Unity component tests has no remaining retained contract gap. See [unity-feature-audit.md](unity-feature-audit.md).
- **Retained art surface** - scenes, materials, art resources, TextMesh Pro, textures, shaders, and metadata remain as inert inputs for future Bevy presentation work.
- **Rust simulation workspace** - the Bevy-free `factory_content`, `factory_sim`, and `factory_cli` crates model the active Unity catalog, finite mining, indexed factories, adjacent inserters and retrievers, manifest and ingredient-driven production, recipes and buffers, programmable destination/item dispatch priorities, multi-hauler arbitration, construction and teardown, deterministic A-star routing and collision arbitration, batteries, bounded per-object alerts, fueled and fuel-free generators, automatic power lines, energy-gated systems, metrics, and a headless runner. See [factory-sim.md](factory-sim.md), [dispatch-policy.md](dispatch-policy.md), [unity-parity.md](unity-parity.md), and [unity-feature-audit.md](unity-feature-audit.md).
- **Iron bars slice** - the starter scenario runs `IronOre -> IronBars` through a deterministic loop that mines a finite deposit at the source, hauls over the road, and crafts at the factory, with JSONL snapshots that expose mining, dispatch, topology, and movement state for debugging.
- **Bevy/Wasm simulation viewer** - `factory_shell` projects immutable simulation snapshots natively and in browsers. It renders topology, movement, inventories, logistics and deployment, structures, production, power, totals, metrics, activity, and the focused object's latest alert. Industrial animation exposes flow between ticks. The showcase cycles through all eleven scenarios, including the five-factory drill chain, separated freight, automatic grid link, warehouse construction, and hybrid generator grid. Cursor movement, bounded camera controls, ten zoom levels, a clickable deck with focus mode, and keyboard actions provide player and playback control. See [factory-viewer.md](factory-viewer.md) and [factory-shell.md](factory-shell.md).
- **Source-owned private web image publication** - Forgejo Actions uses the trusted deploy lane to build the Trunk/Wasm Dockerfile and publish `forgejo.coilysiren.me/coilyco-gaming/factory-game-v3:<full-source-sha>`. The deploy repo owns the read-only pull credential, rollout, and public exposure.
- **Current validation surface** - `ward exec test` runs the repo's pre-commit baseline plus the Rust workspace tests, and CI calls `bash scripts/test-gate.sh` directly for the same check.
- **C# decommission record** - the file-to-Rust proof and preserved asset boundary live in [csharp-decommission.md](csharp-decommission.md).

## See also

- [README.md](../README.md)
- [AGENTS.md](../AGENTS.md)
- [`.ward/ward.yaml`](../.ward/ward.yaml)
- [docs/features-release-tooling.md](features-release-tooling.md)
