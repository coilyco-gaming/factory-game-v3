# Agent instructions

This repo is the factory-game-v3 migration baseline. The current surface is still Unity/C#, but the end state is Rust/Bevy. Treat the Unity code and assets as migration references, not as the final product.

## Scope

- Keep the repo aligned to the Unity-to-Rust/Bevy transition.
- Preserve the current `Assets/` tree unless an issue explicitly says to remove or rewrite it.
- Do not touch `.gitattributes` or binary asset policy here unless the work explicitly includes the LFS migration from issue #4.

## Project shape

- `Assets/` holds the retained Unity-era source, scenes, materials, plugins, and `.meta` files.
- `tests.csproj` is the current executable test surface.
- `README.md`, `AGENTS.md`, `docs/FEATURES.md`, and `.ward/ward.yaml` are the repo-local baseline trio plus command surface.

## Repo boundaries

- Keep the repo aligned to the Unity-to-Rust/Bevy transition.
- Preserve the current `Assets/` tree unless an issue explicitly says to remove or rewrite it.
- Do not touch `.gitattributes` or binary asset policy here unless the work explicitly includes the LFS migration from issue #4.

## Commands

- Route dev commands through ward.
- `ward exec test` is the current repo verb. It runs the repo's pre-commit baseline with a bounded timeout. CI calls `bash scripts/test-gate.sh` directly because repo verbs require a tracked branch.
- Add new verbs to [`.ward/ward.yaml`](.ward/ward.yaml) before using them.
- Do not route work through bare `dotnet` in docs or agent instructions.

## Validation

- Run `ward exec test` for the repo baseline.
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

- This repo is not shipping the Bevy rewrite yet.
- The current task is baseline adoption only.
- Land baseline and workflow changes together so the repo stays internally consistent.

## Agent rules

- Use the current Unity-era surface as migration reference material, not as a target to clean up speculatively.
- Prefer the smallest local exclusion that makes the managed hooks reflect the real repo surface.
- Do not route work through bare `dotnet` in docs or agent instructions.

## See also

- [README.md](README.md)
- [docs/FEATURES.md](docs/FEATURES.md)
- [`.ward/ward.yaml`](.ward/ward.yaml)
- [docs/features-release-tooling.md](docs/features-release-tooling.md)
