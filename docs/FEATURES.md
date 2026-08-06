# Features

What `factory-game-v3` currently ships.

## Inventory

- **Completed Rust gameplay migration** - the C# gameplay tree, test project, and Unity-only dependency plugins are removed. Component contracts are covered by the source audit, while a deterministic 100x100 startup and active run prove the integrated systems at v2 scale. See [unity-feature-audit.md](unity-feature-audit.md) and [v2-world.md](v2-world.md).
- **Retained art surface** - scenes, materials, art resources, TextMesh Pro, textures, shaders, and metadata remain as inert inputs for future Bevy presentation work.
- **Rust simulation workspace** - the Bevy-free `factory_content`, `factory_sim`, and `factory_cli` crates model the active Unity catalog, finite mining, indexed factories, adjacent inserters and retrievers, production, dispatch, multi-hauler arbitration, construction and teardown, cached deterministic A-star routes, batteries, alerts, generators, power lines, energy gates, metrics, and a headless runner. Wide object identities, spatial power balancing, and seeded hauler positions support hundreds of world objects. See [factory-sim.md](factory-sim.md), [dispatch-policy.md](dispatch-policy.md), and [unity-parity.md](unity-parity.md).
- **Iron bars slice** - the starter scenario runs `IronOre -> IronBars` through a deterministic loop that mines a finite deposit at the source, hauls over the road, and crafts at the factory, with JSONL snapshots that expose mining, dispatch, topology, and movement state for debugging.
- **Bevy/Wasm simulation viewer** - `factory_shell` projects immutable simulation snapshots natively and in browsers. It renders topology, movement, inventories, logistics, deployment, production, power, metrics, and activity. The showcase cycles through twelve scenarios, culminating in the 100x100 v2 world. Local and map-overview zoom, accelerated grid panning, a clickable deck, and focus mode keep both micro fixtures and the large world inspectable. See [factory-viewer.md](factory-viewer.md) and [factory-shell.md](factory-shell.md).
- **Source-owned private web image publication** - Forgejo Actions uses the trusted deploy lane to build the Trunk/Wasm Dockerfile and publish `forgejo.coilysiren.me/coilyco-gaming/factory-game-v3:<full-source-sha>`. The deploy repo owns the read-only pull credential, rollout, and public exposure.
- **Current validation surface** - `ward exec test` runs the repo's pre-commit baseline plus the Rust workspace tests, and CI calls `bash scripts/test-gate.sh` directly for the same check.
- **C# decommission record** - the file-to-Rust proof and preserved asset boundary live in [csharp-decommission.md](csharp-decommission.md).

## See also

- [README.md](../README.md)
- [AGENTS.md](../AGENTS.md)
- [`.ward/ward.yaml`](../.ward/ward.yaml)
- [docs/features-release-tooling.md](features-release-tooling.md)
