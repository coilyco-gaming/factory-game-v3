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

The viewer rotates through twelve scenarios, from focused fixtures for freight,
power, building, and production to the 100x100 v2 world. Each scene displays:

- source, road, factory, radar, build-site, structure, generator, and blocked cells
- hauler position, cargo, and dispatch phase
- source and per-factory inventory plus dormant, draining, and exhausted lifecycle
- remote generator type plus coal-site occupancy without false drill activation
- independent craft progress and output item for every factory
- generator mode and fuel, total energy, node batteries, use, and starvation
- radar deployment item, target resource, current claim, and claim activity
- material-flow metrics, per-object alerts, and a bounded rolling activity feed
- global object and stored-resource totals

The scene rebuilds when the scenario changes and interpolates haulers between
authoritative snapshots. Screen-space panels align persistent metrics into
columns and keep ten one-line events outside the world projection. The host
retains the same ten events across scenarios. A presentation offset keeps
labels between those top panels and the compact control deck. No presentation
depends on Unity assets or Git LFS objects. Cursor focus shows the selected object's latest alert in a compact bottom-left overlay.

Material-flow telemetry brightens stocked sources and active factories. Gauges
track craft and grid progress, completed output ejects product chips, active
nodes pulse, route dashes show direction, and loaded cargo badges brighten. The
generators reflect fuel and charge state. Frame-time effects never write back
into simulation state.

Automatic cycling advances to the next scenario after eight consecutive quiet
ticks. This gives the native and browser viewers a continuous demonstration
without changing deterministic simulation state. The quiet-tick threshold is
viewer presentation policy only.

## Controls

The control deck works with mouse or touch in native and web builds. It provides
play or pause, single-step, reset, speed, automatic cycle, and starter-scenario
buttons. `HIDE UI` enters focus mode by hiding HUD and event text, labels,
focus cursor, badges, and gauges, then collapses the deck to a `SHOW UI` restore
button. Active scenario and toggle buttons stay highlighted.

The keyboard mirrors the same typed actions:

- `Space` toggles play and pause.
- `N` pauses and advances one deterministic tick.
- `R` resets the scenario.
- `F` toggles between 2 and 8 ticks per second.
- `C` advances to the next scenario.
- `L` toggles automatic scenario cycling.
- `WASD` or arrow keys move focus. Hold `Shift` to move ten cells.
- `Q`/`E` or the wheel zoom. `O` toggles detail and whole-map views.

## Development

- `ward exec shell-run` starts the native viewer.
- `ward exec shell-serve` starts the browser viewer with Trunk hot reload.
- `ward exec shell-build-web` builds the static Wasm bundle.
- `ward exec cargo-test` proves hosted ticks match direct simulation.
