# Factory simulation viewer

The Bevy viewer makes the deterministic Rust simulation visible without
moving any simulation authority into Bevy. It lives in `crates/factory_shell`
so the repository keeps one native and web application surface.

## Authority boundary

`factory_sim::GameState` owns mining, dispatch, movement, inventory,
production, metrics, and tick ordering. The viewer owns only:

- playback cadence and controls
- projection of immutable `TickSnapshot` values
- node, route, hauler, and text presentation
- native-window and Wasm runtime behavior

Bevy frames and simulation ticks use separate clocks. The viewer advances the
simulation in whole ticks at a bounded cadence and projects the latest
snapshot after each update. It does not interpolate or mutate simulation state
through Bevy entities.

## First viewer slice

The initial viewer runs the `iron-bars` scenario and displays:

- source, road, and factory topology nodes
- hauler position, cargo, and dispatch phase
- source and factory inventory
- factory craft progress
- current tick, run metrics, and recent events

The scene uses generated primitives and Bevy text. It does not depend on
retained Unity assets or Git LFS objects.

## Controls

- `Space` toggles play and pause.
- `N` pauses and advances one deterministic tick.
- `R` resets the scenario.
- `F` toggles between 2 and 8 ticks per second.

## Development

- `ward exec shell-run` starts the native viewer.
- `ward exec shell-serve` starts the browser viewer with Trunk hot reload.
- `ward exec shell-build-web` builds the static Wasm bundle.
- `ward exec cargo-test` proves the viewer-hosted tick stream matches direct
  simulation stepping.

## Deferred

Scenario selection, movement interpolation, camera controls, production art,
recording playback, and gameplay editing remain outside the first viewer
slice. The viewer is an observability surface before it becomes a game client.
