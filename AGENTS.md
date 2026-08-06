# Agent instructions

This repo is the factory-game-v3 migration baseline. The current surface is still Unity/C#, but the end state is Rust/Bevy. Treat the Unity code and assets as migration references, not as the final product.

## Scope

- Keep the repo aligned to the Unity-to-Rust/Bevy transition.
- Preserve the current `Assets/` tree unless an issue explicitly says to remove or rewrite it.
- Do not touch `.gitattributes` or binary asset policy here unless the work explicitly includes the LFS migration from issue #4.

## Project shape

- `Assets/` holds the retained Unity-era source, scenes, materials, plugins, and `.meta` files.
- `tests.csproj` is a retained Unity-era reference project, not the active validation surface.
- `crates/` holds the active Rust workspace for the first migration slices, while Unity code remains as reference material.
- `README.md`, `AGENTS.md`, `docs/FEATURES.md`, and `.ward/ward.yaml` are the repo-local baseline trio plus command surface.

## Repo boundaries

- Keep the repo aligned to the Unity-to-Rust/Bevy transition.
- Preserve the current `Assets/` tree unless an issue explicitly says to remove or rewrite it.
- Do not touch `.gitattributes` or binary asset policy here unless the work explicitly includes the LFS migration from issue #4.

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
- `ward exec test` is the current repo verb. It runs the repo's pre-commit baseline with a bounded timeout. CI calls `bash scripts/test-gate.sh` directly because repo verbs require a tracked branch.
- Add new verbs to [`.ward/ward.yaml`](.ward/ward.yaml) before using them.
- Do not route work through bare `dotnet` in docs or agent instructions.

## Validation

- Run `ward exec test` for the repo baseline, including the Rust workspace.
- Run `ward exec image-build` when changing the web image or publish workflow.
- Run `ward exec check-publish` when changing the Forgejo OCI publisher.
- Run `pre-commit run --all-files` before committing.
- Keep [README.md](README.md), this file, and [docs/FEATURES.md](docs/FEATURES.md) in sync when the repo surface changes.

## Safety

- Keep tracked text public-safe.
- Keep repo-local baseline exceptions narrow and documented in the lowest config layer that can express them.
- Treat Unity asset retention as a migration constraint, not a license to normalize the tree wholesale.

## Cross-repo contracts

- Keep baseline exclusions narrow and documented.
- Coordinate binary asset policy changes with issue #4.
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

- Use the current Unity-era surface as migration reference material, not as a target to clean up speculatively.
- Prefer the smallest local exclusion that makes the managed hooks reflect the real repo surface.
- Do not route work through bare `dotnet` in docs or agent instructions.

## See also

- [README.md](README.md)
- [docs/FEATURES.md](docs/FEATURES.md)
- [`.ward/ward.yaml`](.ward/ward.yaml)
- [docs/features-release-tooling.md](docs/features-release-tooling.md)
