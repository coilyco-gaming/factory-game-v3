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

Bevy frames and simulation ticks use separate clocks. Snapshot revisions gate
world and HUD projection, while hauler interpolation and cached node activity
animate between ticks without rescanning simulation collections. Overview hides
unreadable object text, badges, and gauges. The three closest zoom levels
restore them, while geometry, focus, and screen status remain visible throughout.

## World roster

The viewer opens directly into the 50x50 v2 world. Focused freight, power,
building, pathfinding, and production fixtures remain available to the CLI and
test suite, but do not appear in the player-facing UI. Each world displays:

- source, road, factory, radar, build-site, structure, generator, and blocked cells
- hauler position, cargo, and dispatch phase
- source and per-factory inventory plus dormant, draining, and exhausted lifecycle
- remote generator type plus coal-site occupancy without false drill activation
- independent craft progress and output item for every factory
- generator mode and fuel, total energy, node batteries, use, and starvation
- radar deployment item, target resource, current claim, and claim activity
- resource stockpiles, material stockpiles, and power metrics

The scene rebuilds when the scenario changes and interpolates haulers between
authoritative snapshots. One full-width top bar stacks Resources, Materials,
and Power as three full-width information rows beside the title. Resources are
ingredientless catalog items, while Materials are recipe-produced items. Both
rows count current stockpiles across the world. One compact control panel sits
at bottom right. Recent activity and focused alert overlays are intentionally
absent. A presentation offset keeps labels between the two screen-edge
surfaces. See [factory-art.md](factory-art.md).

Material-flow telemetry brightens stocked sources and active factories. Gauges
track craft and grid progress, completed output ejects product chips, active
nodes pulse, route dashes show direction, and loaded cargo badges brighten. The
generators reflect fuel and charge state. Frame-time effects never write back
into simulation state.

## Controls

The bottom-right control panel works with mouse or touch in native and web
builds. It provides play or pause, single-step, reset, and speed controls.
`HIDE UI` hides the top bar, labels, focus cursor, badges, and gauges, then
collapses the panel to a `SHOW UI` restore button. Active toggles stay
highlighted.

The keyboard mirrors the same typed actions:

- `Space` toggles play and pause.
- `N` pauses and advances one deterministic tick.
- `R` resets the scenario.
- `F` toggles between 2 and 8 ticks per second.
- `C` advances to the next world when the roster contains more than one.
- `WASD` or arrows move focus and repeat while held. `Shift` moves ten cells.
- `Q`/`E` zoom and repeat while held. The wheel steps once. `O` toggles the
  approximately 10x10-cell detail and full-world views. Startup fits the world
  and cannot zoom farther out.

## Development

- `ward exec shell-run` starts the native viewer with an interactive profile
  that optimizes Bevy and its graphics dependencies.
- `ward exec shell-serve` starts the browser viewer with Trunk hot reload.
- `ward exec shell-build-web` builds the static Wasm bundle.
- `ward exec cargo-test` proves hosted ticks match direct ticks.
