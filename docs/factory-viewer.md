# Factory planning surface

The Bevy native and Wasm app is the player-facing compact freight yard. It
projects immutable `CompactSnapshot` values and sends explicit edit commands
to `CompactGame`. Simulation rules remain Bevy-free.

## World

The app opens paused and fitted to the full provisional 16x16 map. The map has
one warehouse/export hub, four visible iron and copper deposits, three trucks,
and a small starter road apron. The former 50x50 worlds remain headless
simulation fixtures and are no longer player-facing scenarios.

The world projection shows:

- every ground and authored road cell
- deposit stock and remaining ore
- placed factories, recipes, input, output, and selection
- warehouse position plus market state in the status bar
- truck cargo and upcoming road route

The top status bar and bottom-right planner are the only screen-space UI
surfaces. Resource rows use available tiny local sprites and literal `//`
separators. The market row shows current demand, cumulative sales, and revenue.

## Planning controls

The planner exposes four unambiguous pointer modes:

- Inspect selects a factory or reports the clicked cell.
- Road paints free road cells while the mouse or touch is held.
- Erase removes free road cells while the mouse or touch is held.
- Factory places one building against the current allowance and selects it.

After selecting a factory, Iron Bars and Copper Bars assign its recipe. The
same panel provides play/pause, one tick, reset, speed, and zoom buttons. This
makes the complete planning loop usable by mouse or touch without keyboard
requirements. Invalid edits return an actionable message in the panel.

Keyboard mirrors the pointer controls: `1` through `4` select tools, `I` and
`C` assign recipes, `Space` plays or pauses, `N` steps, `R` resets, and `F`
changes speed. `WASD` or arrows pan while held. `Q`, `E`, and the wheel zoom.
The closest view shows about 10x10 cells, while the maximum zoom-out remains
the complete world extent.

## Frame pacing

The native shell redraws continuously only while the simulation runs. Paused,
it switches to winit's reactive mode and idles until input arrives, because a
paused window otherwise ran the full render loop forever and held a core at
roughly 90 percent while nothing advanced. The shell opens paused, so it opens
reactive, and resuming flips it back to continuous frames rather than leaving
a running simulation waiting on events.

The browser build stays continuous. A DOM click is not a winit event, so a
sleeping web build ignores the accessible panel until a mouse moves over the
canvas. See [accessible-play.md](accessible-play.md).

## Projection performance

The 256-cell ground is static. Snapshot revisions rebuild only the compact
dynamic layer of roads, deposits, factories, routes, labels, and three trucks.
Truck transforms interpolate between authoritative cells. No presentation
system writes simulation state directly.
