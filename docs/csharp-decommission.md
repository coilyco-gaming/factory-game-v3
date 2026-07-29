# C# decommission batches

The Rust migration removes retained C# only after deterministic tests cover its
state transitions and the viewer exposes player-visible state.

## Batch one: movement and deployment

Rust now covers the retained behavior in:

- `Assets/Scripts/Components/PathfindingComponent.cs`
- `Assets/Scripts/Components/MovementComponent.cs`
- `Assets/Scripts/Components/DeploymentComponent.cs`
- the corresponding truck wiring in `WorldObject.cs` and
  `WorldObjects/WorldObjectTruck.cs`

Deterministic simulation tests cover path success and failure, obstacle detours,
queued movement, transit arbitration, retrieve and deploy transitions,
depleted-ore deletion, and drill teardown. The Bevy viewer exposes each
player-visible state through DETOUR and DEPLOY.

This is the first safe removal batch. The agent removes the three component
files, their `.meta` files, and only their direct truck and core fields together.
Mining, dispatch, resources, production, and power C# remain reference material
until their wider object and content dependencies reach the same proof boundary.

## Final deletion gate

The repository removes all of `Assets/Scripts/`, `tests.csproj`, and C#-only
plugins only after every remaining gameplay component reaches a proven Rust
replacement or an explicit obsolete decision. Retained visual assets follow a
separate decision.
