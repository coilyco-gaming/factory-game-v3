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

### 2026-07-29 - general structure construction

- Done - scenarios can declare typed build sites. Factories advertise matching
  spawnable inventory, haulers retrieve one item, and deployment queues a world
  mutation that replaces the site with a blocked structure node.
- Evidence - the warehouse scenario begins without resource sources, transports
  the Unity-catalog warehouse under its real weight and volume constraints, and
  exposes the spawned structure in snapshots and the tenth viewer control.
- Next - run the full gate, wait for Wasm hot reload, land, then audit general
  targeting and buffer-policy parity.

### 2026-07-29 - construction validation

- Evidence - the full gate and all 52 Rust tests pass. The proof asserts the
  open build site, factory retrieval, single spawn event, typed structure
  snapshot, occupied replacement cell, consumed inventory, and no duplication.
- Evidence - Trunk applied the tenth viewer scene and the live BUILD control now
  shows the full warehouse trip and topology replacement.
- Next - land and audit Unity's configurable dispatch target and buffer policy.

### 2026-07-29 - dispatch policy audit

- Evidence - Unity resolves non-`Me` dispatch objects by matching world-object
  type, ordering matches by distance from the dispatcher, rejecting an already
  claimed deploy target, then choosing the nearest compatible free receiver.
- Decision - Rust keeps typed endpoints, resolves providers and deployment
  targets by squared grid distance with node-id tie-breaking, and retains its
  safer scenario-owned per-factory unit buffer instead of Unity's fixed
  four-stack constant.
- Evidence - focused tests prove nearest matching source selection, nearest
  unoccupied drill target selection, and a configured buffer acting as the
  exact intent threshold. All 55 existing and new Rust tests pass.
- Next - run the full gate, land, then audit player placement and selection.

### 2026-07-29 - dispatch policy validation

- Evidence - the full gate passes with all 55 Rust tests.
- Next - land and begin the player interaction audit.
