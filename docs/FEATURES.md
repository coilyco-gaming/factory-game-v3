# Features

What `factory-game-v3` currently ships.

## Inventory

- **Completed Rust gameplay migration** - the C# tree, test project, Unity-only plugins, and the retained Unity asset tree are gone, with deterministic v2-scale runs standing in for the old component contracts. See [unity-feature-audit.md](unity-feature-audit.md), [v2-world.md](v2-world.md), and [unity-parity.md](unity-parity.md).
- **Rust simulation workspace** - `factory_content`, `factory_sim`, and `factory_cli` own the catalog, the authoritative rules, and the headless runner: mining, factories, inserters, dispatch, multi-hauler arbitration, construction, cached A-star routes, batteries, generators, power lines, alerts, and metrics. See [factory-sim.md](factory-sim.md), [headless-runner.md](headless-runner.md), [factory-power.md](factory-power.md), and [dispatch-policy.md](dispatch-policy.md).
- **Compact first-playable simulation** - a separate authoritative 16x16 state models a warehouse, four ore deposits, editable roads, road-frontage placement, iron and copper recipes, truck logistics, rising market demand, sales, and unlockable building allowance. See [compact-first-playable.md](compact-first-playable.md).
- **Session persistence** - `factory_sim` owns a versioned save format that round-trips `CompactGame` through a string, and the shell parks that string in browser `localStorage` or a native file without reading it. See [compact-persistence.md](compact-persistence.md).
- **Bevy/Wasm planning surface** - `factory_shell` projects the compact loop natively and in browsers, with mouse, touch, and keyboard control over roads, factories, recipes, time, pan, and zoom. See [factory-viewer.md](factory-viewer.md) and [factory-shell.md](factory-shell.md).
- **Runtime factory art** - fifteen 100px RGBA sprites ship in ordinary Git under `crates/factory_shell/assets/factory/`, render on matching identities, and fall back to colored shapes where no sprite exists. Trunk, the image, and nginx carry them to the browser. See [factory-art.md](factory-art.md).
- **Deployment radar authority** - fixture-owned radars discover dormant deposits, hold exclusive claims, release invalid ones, and delegate retrieval to factories. See [deployment-radar.md](deployment-radar.md).
- **Remote coal-plant construction** - a radar claims a free coal deposit and a queued mutation places a plant with its own storage, battery, and grid link. See [remote-coal-plants.md](remote-coal-plants.md).
- **Sustained v2 operation** - a deterministic two-replay 650-tick proof drains every non-generator battery at tick 500 and keeps extraction, freight, production, and generation running. See [v2-liveness.md](v2-liveness.md).
- **Iron bars slice** - the starter scenario runs `IronOre -> IronBars` end to end and emits JSONL snapshots of mining, dispatch, topology, and movement. See [factory-scenarios.md](factory-scenarios.md).
- **Source-owned private web image publication** - Forgejo Actions builds the Trunk/Wasm Dockerfile on the trusted deploy lane and publishes `forgejo.coilysiren.me/coilyco-gaming/factory-game-v3:<full-source-sha>`. The deploy repo owns the pull credential, rollout, and exposure. See [features-release-tooling.md](features-release-tooling.md).
- **Current validation surface** - `ward exec test` runs the pre-commit baseline plus the Rust workspace tests, and CI calls `bash scripts/test-gate.sh` for the same check.
- **C# decommission record** - the file-to-Rust proof and the fate of the Unity asset tree live in [csharp-decommission.md](csharp-decommission.md).

## See also

- [README.md](../README.md)
- [AGENTS.md](../AGENTS.md)
- [`.ward/ward.yaml`](../.ward/ward.yaml)
- [docs/features-release-tooling.md](features-release-tooling.md)
