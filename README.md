# Factory Game V3

This repository ships the Rust/Bevy factory simulation and browser viewer.
Reusable art from the retired Unity prototype remains available for future
presentation work.

## Validation

Run the lightweight migration baseline through ward:

```bash
ward exec test
```

That verb runs the repo's pre-commit baseline and Rust workspace tests with a
bounded timeout. CI calls the same bounded script directly because ward repo
verbs require a tracked branch.

The headless Rust slice can also run directly:

```bash
ward exec cargo-run -- run --scenario iron-bars --ticks 6
```

## Deployment

Source CI builds the repo-root Dockerfile and publishes
`forgejo.coilysiren.me/coilyco-gaming/factory-game-v3:<full-source-sha>` as a
private Forgejo package on every push to `main`.
The image packages the Trunk-built Wasm bundle behind unprivileged nginx.
`coilyco-bridge/deploy` owns the public chart, rollout, and its separate
read-only `forgejo-registry` pull credential.

Validate the image locally through Ward:

```bash
ward exec check-publish
ward exec image-build
```

## Inventory

See [docs/FEATURES.md](docs/FEATURES.md) for the current feature inventory and migration surface.
See [docs/unity-parity.md](docs/unity-parity.md) for the source-level gameplay
audit, [docs/v2-world.md](docs/v2-world.md) for the game-scale proof, and
[docs/deployment-radar.md](docs/deployment-radar.md) for autonomous target
claims. Remote power expansion is covered in
[docs/remote-coal-plants.md](docs/remote-coal-plants.md), and the post-starter
power proof is in [docs/v2-liveness.md](docs/v2-liveness.md).

## See also

- [AGENTS.md](AGENTS.md)
- [docs/FEATURES.md](docs/FEATURES.md)
- [.ward/ward.yaml](.ward/ward.yaml)
- [docs/features-release-tooling.md](docs/features-release-tooling.md)
