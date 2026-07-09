# Factory Game V3

This repository is in transition from Unity/C# to Rust/Bevy. Unity project settings and editor/package scaffolding have been removed. Gameplay source and reusable assets remain as migration references.

## Validation

Run the current test surface through ward:

```bash
ward exec test
```

That verb runs the repo's pre-commit baseline with a bounded timeout. CI calls the same bounded script directly because ward repo verbs require a tracked branch.

## Inventory

See [docs/FEATURES.md](docs/FEATURES.md) for the current feature inventory and migration surface.

## See also

- [AGENTS.md](AGENTS.md)
- [docs/FEATURES.md](docs/FEATURES.md)
- [.ward/ward.yaml](.ward/ward.yaml)
- [docs/features-release-tooling.md](docs/features-release-tooling.md)
