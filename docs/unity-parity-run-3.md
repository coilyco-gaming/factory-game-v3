# Unity parity unattended run, part 3

Contract: [Forgejo issue 37](https://forgejo.coilysiren.me/coilyco-gaming/factory-game-v3/issues/37)

### 2026-07-29 - automatic power-line audit

- Evidence - Unity power-line objects are pass-through batteries with capacity
  1,000. A generator performs one greedy eight-neighbor route toward the nearest
  other energized line endpoint and queues every missing line cell.
- Decision - Rust will preserve one-time nearest-target greedy construction and
  explicit line batteries, while selecting the nearest disconnected battery as
  the useful bootstrap target.
- Next - expose a separated powered layout whose generated line visibly joins
  the coal plant to the consumer network.

### 2026-07-29 - automatic grid link

- Done - generators perform one deterministic nearest-battery route and create
  every missing eight-neighbor line cell before energy balancing.
- Evidence - the grid-link scenario builds exactly three pass-through cells,
  gives each a 1,000-capacity battery, energizes the remote factory, and exposes
  the route in snapshots and the ninth viewer control.
- Correction - coal-plant tuning now matches Unity's 10,000 battery capacity,
  four-unit burn rate, and 160-unit generation rate.
- Next - validate, wait for the Wasm scene, land, then port general world
  spawning and spawnable factory outputs.

### 2026-07-29 - grid-link validation

- Evidence - the full gate and all 51 Rust tests pass. Trunk applied the ninth
  scenario with visible line cells and remote node charge.
- Decision - power behavior is now complete enough to leave only general world
  construction and player interaction in the critical migration path.
- Next - land and begin spawnable output construction.
