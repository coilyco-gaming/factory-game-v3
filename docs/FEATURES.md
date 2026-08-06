# Features

What `factory-game-v3` currently ships.

## Inventory

- **Completed Rust gameplay migration** - the C# gameplay tree, test project, and Unity-only dependency plugins are removed. Component contracts are covered by the source audit, while deterministic 50x50 startup, active-flow, and post-starter-power runs prove the integrated systems at v2 scale. See [unity-feature-audit.md](unity-feature-audit.md), [v2-world.md](v2-world.md), and [v2-liveness.md](v2-liveness.md).
- **Retained art surface** - scenes, materials, art resources, TextMesh Pro, textures, shaders, and metadata remain as inert inputs for future Bevy presentation work.
- **Local runtime factory art** - fifteen accepted 100px RGBA sprites live under `crates/factory_shell/assets/factory/` in ordinary Git. A typed Bevy resource renders them on matching terrain, logistics, deposit, machine, structure, and cargo identities without placeholder backplates, while unsprited identities retain colored fallbacks. Interpolation, labels, gauges, route effects, and overview performance remain intact. See [factory-art.md](factory-art.md).
- **Rust simulation workspace** - the Bevy-free `factory_content`, `factory_sim`, and `factory_cli` crates model the active Unity catalog, finite mining, indexed factories, adjacent inserters and retrievers, production, dispatch, multi-hauler arbitration, construction and teardown, cached deterministic A-star routes, batteries, alerts, generators, ordered generator-owned power lines, energy gates, per-generator and aggregate metrics, and a headless runner. Wide object identities, indexed topology positions, spatial power balancing, snapshot-free stepping, and bounded liveness summaries support hundreds of world objects. See [factory-sim.md](factory-sim.md), [headless-runner.md](headless-runner.md), [factory-power.md](factory-power.md), [dispatch-policy.md](dispatch-policy.md), and [unity-parity.md](unity-parity.md).
- **Deployment radar authority** - scenario-owned radars discover compatible dormant deposits, retain exclusive claims, release invalid or completed claims, and delegate inventory retrieval to factories. Snapshots, events, per-radar alerts, CLI output, and both viewer zoom modes expose the targeting lifecycle. See [deployment-radar.md](deployment-radar.md).
- **Remote coal-plant construction** - the fourth v2 radar claims an unoccupied coal deposit for the retained coal-plant recipe. Normal factory inventory, typed retrieve/deploy phases, and a queued generator mutation place a uniquely identified plant with coal storage, its own battery, automatic grid linking, snapshots, metrics, events, and viewer state. See [remote-coal-plants.md](remote-coal-plants.md).
- **Sustained v2 operation** - a deterministic two-replay, 650-tick release proof zeros every non-generator battery at tick 500. Generator-owned extensions reconnect drained active deposits, while remote extraction, freight, upper-tier production, coal delivery, and generator output continue under structural liveness bounds. See [v2-liveness.md](v2-liveness.md).
- **Iron bars slice** - the starter scenario runs `IronOre -> IronBars` through a deterministic loop that mines a finite deposit at the source, hauls over the road, and crafts at the factory, with JSONL snapshots that expose mining, dispatch, topology, and movement state for debugging.
- **Bevy/Wasm world viewer** - `factory_shell` projects immutable simulation snapshots natively and in browsers. It renders topology, movement, inventories, logistics, radar claims, deployment, production, power, metrics, and matching local art. A full-width status bar and bottom-right control panel are the only screen-space UI surfaces. The camera starts fitted to the full 50x50 extent, cannot zoom farther out, and retains an approximately 10x10-cell close view. Held-key navigation and focus mode keep the world inspectable. See [factory-viewer.md](factory-viewer.md) and [factory-shell.md](factory-shell.md).
- **Source-owned private web image publication** - Forgejo Actions uses the trusted deploy lane to build the Trunk/Wasm Dockerfile and publish `forgejo.coilysiren.me/coilyco-gaming/factory-game-v3:<full-source-sha>`. The deploy repo owns the read-only pull credential, rollout, and public exposure.
- **Current validation surface** - `ward exec test` runs the repo's pre-commit baseline plus the Rust workspace tests, and CI calls `bash scripts/test-gate.sh` directly for the same check.
- **C# decommission record** - the file-to-Rust proof and preserved asset boundary live in [csharp-decommission.md](csharp-decommission.md).

## See also

- [README.md](../README.md)
- [AGENTS.md](../AGENTS.md)
- [`.ward/ward.yaml`](../.ward/ward.yaml)
- [docs/features-release-tooling.md](features-release-tooling.md)
