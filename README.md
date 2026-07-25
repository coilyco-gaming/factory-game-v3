# Factory Game V3

This repository is in transition from Unity/C# to Rust/Bevy. Unity project settings and editor/package scaffolding have been removed. Gameplay source and reusable assets remain as migration references.

## Validation

Run the lightweight migration baseline through ward:

```bash
ward exec test
```

That verb runs the repo's pre-commit baseline and the Rust workspace tests with a bounded timeout. CI calls the same bounded script directly because ward repo verbs require a tracked branch. The retained `tests.csproj` project is reference-only and is not part of the active gate.

The headless Rust slice can also run directly:

```bash
ward exec cargo-run -- run --scenario iron-bars --ticks 6
```

## Deployment

Source CI builds the repo-root Dockerfile and publishes
`factory-game-v3:<git-sha>` to the in-cluster registry on every push to `main`.
The image packages the Trunk-built Wasm bundle behind unprivileged nginx.
`coilyco-bridge/deploy` owns the public chart and rollout.

Validate the image locally through Ward:

```bash
ward exec image-build
```

## Inventory

See [docs/FEATURES.md](docs/FEATURES.md) for the current feature inventory and migration surface.

## See also

- [AGENTS.md](AGENTS.md)
- [docs/FEATURES.md](docs/FEATURES.md)
- [.ward/ward.yaml](.ward/ward.yaml)
- [docs/features-release-tooling.md](docs/features-release-tooling.md)
