# Features

What `factory-game-v3` currently ships. This repo is still in migration, so this inventory describes the present Unity/C# reference surface and retained assets.

## Inventory

- **Unity/C# migration reference state** - Unity project settings and editor/package scaffolding are gone, but the repo still holds the current Unity-era gameplay reference surface while the Rust/Bevy rewrite is staged.
- **Gameplay and domain code** - `Assets/Scripts/` contains the current core model, controller, world-object, and utility code that future Bevy work will replace or port.
- **Retained art and asset surface** - `Assets/Scenes/`, `Assets/Materials/`, `Assets/Plugins/`, `Assets/TextMesh Pro/`, and the remaining `.meta` files stay in place as migration references.
- **Current validation surface** - `ward exec test` runs the repo's pre-commit baseline with a bounded timeout. `tests.csproj` remains as the retained C# migration reference, but it is not the CI gate.

## See also

- [README.md](../README.md)
- [AGENTS.md](../AGENTS.md)
- [`.ward/ward.yaml`](../.ward/ward.yaml)
- [docs/features-release-tooling.md](features-release-tooling.md)
