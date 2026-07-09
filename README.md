# Factory Game V3

This repository is in transition from Unity/C# to Rust/Bevy. Unity project settings and editor/package scaffolding have been removed. Gameplay source and reusable assets remain as migration references.

## Validation

Run the current test surface through ward:

```bash
ward exec test
```

That verb wraps the existing `tests.csproj` xUnit suite.

## Inventory

See [docs/FEATURES.md](docs/FEATURES.md) for the current feature inventory and migration surface.

## See also

- [AGENTS.md](/workspace/factory-game-v3/AGENTS.md)
- [docs/FEATURES.md](/workspace/factory-game-v3/docs/FEATURES.md)
- [.ward/ward.yaml](/workspace/factory-game-v3/.ward/ward.yaml)
- [docs/features-release-tooling.md](/workspace/factory-game-v3/docs/features-release-tooling.md)
