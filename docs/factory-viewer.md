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

Bevy frames and simulation ticks use separate clocks. The viewer advances the simulation in whole ticks at a bounded cadence and projects the latest
snapshot after each update. Bevy smoothly interpolates only the visual hauler
projection between snapshots. That animation never mutates simulation state
through Bevy entities.

## Showcase

The viewer rotates through six scenarios, including powered ironworks, drill
deployment, and a collision-arbitrated obstacle convoy. Each scene displays:

- active or awaiting-drill source, road, factory, power-plant, and blocked cells
- hauler position, cargo, and dispatch phase
- source and factory inventory
- factory craft progress
- coal-plant fuel, power-grid energy, generation, consumption, and starvation
- current tick, run metrics, and a bounded rolling activity feed

The scene rebuilds its generated primitives when the scenario changes. Hauler
sprites move smoothly between the discrete positions in each authoritative
snapshot. The presentation does not depend on retained Unity assets or Git
LFS objects.

Material-flow telemetry brightens stocked sources and changes the factory color
while it demands inputs or crafts. A slim gauge tracks recipe progress, and
completed output ejects a short stack of stone-and-steel product chips. Active
nodes pulse, route dashes show direction, and cargo badges brighten when loaded.
The coal plant changes state as it burns fuel or holds charge. Its gauge tracks the authoritative shared grid.
The latest eight events persist across scenarios. Frame-time effects move
smoothly between ticks and never write back into simulation state.

Automatic cycling advances to the next scenario after eight consecutive quiet
ticks. This gives the native and browser viewers a continuous demonstration
without changing deterministic simulation state. The quiet-tick threshold is
viewer presentation policy only.

## Controls

The screen-space control deck works with a mouse or touch in native and web
builds. It provides play or pause, single-step, reset, speed, automatic cycle,
and direct buttons for every starter scenario. Active scenario and toggle
buttons stay highlighted.

The keyboard mirrors the same typed actions:

- `Space` toggles play and pause.
- `N` pauses and advances one deterministic tick.
- `R` resets the scenario.
- `F` toggles between 2 and 8 ticks per second.
- `C` advances to the next scenario.
- `L` toggles automatic scenario cycling.

## Development

- `ward exec shell-run` starts the native viewer.
- `ward exec shell-serve` starts the browser viewer with Trunk hot reload.
- `ward exec shell-build-web` builds the static Wasm bundle.
- `ward exec cargo-test` proves the viewer-hosted tick stream matches direct
  simulation stepping.

## Deferred

Scenario editing, camera controls, production art, recording playback, and
gameplay editing remain deferred. The viewer is an observability surface
before it becomes a game client.
