# Agent instructions

This repo is the factory-game-v3 Rust/Bevy factory game. The Unity-to-Rust migration is complete: the C# surface and the Unity asset tree are both gone, recorded in [docs/csharp-decommission.md](docs/csharp-decommission.md). Runtime art the game actually uses lives in the crate that renders it.

## Scope

- The Rust workspace under `crates/` is the product. Build features there.
- Art the game renders belongs to the crate that renders it, under `crates/factory_shell/assets/`. Do not reintroduce an engine-shaped asset tree at the repo root.
- `.gitattributes` sends large binary art to LFS and keeps the small runtime sprites in ordinary Git so the viewer builds from a plain clone. Change it only when an issue says to.

## Project shape

- `crates/` holds the Rust workspace: `factory_content` (catalog), `factory_sim` (authority), `factory_cli` (headless runner), `factory_shell` (Bevy viewer, native and Wasm).
- `scripts/` holds the test gate, the Trunk web build, and the trusted image publisher. `Dockerfile` and `nginx.conf` build and serve the browser bundle.
- `README.md`, `AGENTS.md`, `docs/FEATURES.md`, and `.ward/ward.yaml` are the repo-local baseline trio plus command surface.

## Repo boundaries

- Keep gameplay and its art in `crates/`. The repo root carries build, deploy, and docs, not game content.
- `coilyco-bridge/deploy` owns rollout, the pull credential, and public exposure. This repo owns the image and the workflow that publishes it.

### The simulation owns logic

New gameplay features are implemented in the simulation and CLI layer. The
presentation layer does not get its own branch of logic.

- `factory_sim` owns authority. Every rule, state transition, and edit validation lands here.
- `factory_cli` exercises those rules headlessly. A feature the runner cannot reach is a feature no determinism proof covers.
- `factory_shell` projects immutable snapshots and sends explicit edit commands back. It decides how the world looks, never what the world does.

State flows one way. The sim produces a snapshot, the shell draws it, the shell
sends a command, the sim decides. A snapshot is never fed back in as input, and
a snapshot type is never a save format or a source of truth.

When a feature needs a new rule, the rule lands in the sim and the shell learns
to draw it. Never the reverse. Logic that lives only in the shell is invisible
to `ward exec cargo-run` and to every `factory_sim` test, which is where this
repo keeps its proofs.

## Commands

- Route dev commands through ward.
- `ward exec test` is the gate: the pre-commit baseline plus the Rust workspace tests, under a bounded timeout. CI calls `bash scripts/test-gate.sh` directly because repo verbs require a tracked branch.
- `ward exec cargo-run` runs a headless scenario, `play` the compact loop over line-oriented JSON, `shell-run` the native viewer, `shell-serve` and `shell-build-web` the browser bundle, `image-build` and `check-publish` the deploy surface, `v2-liveness` the sustained-operation proof.
- Enumerate the full set in [`.ward/ward.yaml`](.ward/ward.yaml), and add a verb there before invoking it.

## Validation

- Run `ward exec test` for the repo baseline, including the Rust workspace.
- Run `ward exec image-build` when changing the web image or publish workflow.
- Run `ward exec check-publish` when changing the Forgejo OCI publisher.
- Run `pre-commit run --all-files` before committing.
- Keep [README.md](README.md), this file, and [docs/FEATURES.md](docs/FEATURES.md) in sync when the repo surface changes.

## Safety

- Keep tracked text public-safe.
- Keep repo-local baseline exceptions narrow and documented in the lowest config layer that can express them.

## Cross-repo contracts

- Keep baseline exclusions narrow and documented.
- The catalog pre-commit hooks are authored in `coilyco-flight-deck/agentic-os`. Fix a validator there rather than working around it here.
- Use the shared agentic-os conventions when the repo adopts a new managed-repo surface.

## Release

- Source CI publishes the private Bevy/Wasm web image to
  `forgejo.coilysiren.me/coilyco-gaming/factory-game-v3:<full-source-sha>`.
- The trusted deploy runner supplies the package write credential as
  `REGISTRY_TOKEN`.
- `coilyco-bridge/deploy` owns rollout, the read-only `forgejo-registry`
  pull secret, and the public ingress.
- Land image and workflow changes together so the deploy contract stays complete.

## Agent rules

- Prefer the smallest local exclusion that makes the managed hooks reflect the real repo surface.
- Bump `COMPACT_SAVE_VERSION` in the same commit that changes the compact save shape. See [docs/compact-persistence.md](docs/compact-persistence.md).

## See also

- [README.md](README.md)
- [docs/FEATURES.md](docs/FEATURES.md)
- [`.ward/ward.yaml`](.ward/ward.yaml)
- [docs/features-release-tooling.md](docs/features-release-tooling.md)
