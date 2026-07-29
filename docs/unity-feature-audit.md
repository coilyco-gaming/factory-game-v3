# Unity feature audit

This audit maps every retained Unity gameplay subsystem to its Rust replacement
status. “Working” means the Rust path has deterministic tests and, where the
behavior is visual, a scenario exposed by the Bevy/Wasm viewer.

* **Content catalog** - working - `factory_content` owns all active resources,
  products, spawnable recipes, physical values, timing, multipliers, manifest
  flags, and spawn flags from `FactoryGameContent.cs`.
* **Resource containers** - working - Rust owns capacity, weight, volume,
  reservations, insertion, removal, transfer, eight-neighbor automated
  insertion, and adjacent receiver retrieval from source or factory containers.
* **Mining** - working - Rust owns finite and manifest extraction, deployment
  requirements, power gating, depletion, and source/drill teardown.
* **Production** - working - Rust owns indexed factories, multi-input recipes,
  progress, output capacity, power gating, and an end-to-end five-factory
  mining-drill chain.
* **Dispatch** - working - Rust owns collect, deliver, retrieve, deploy,
  fleet arbitration, in-flight work, one-phase-per-tick receiver transitions,
  fuel delivery, factory-output supply, nearest matching world-object
  resolution, per-factory buffer thresholds, duplicate-target exclusion, and
  occupied-target avoidance.
* **Movement and pathfinding** - working - Rust owns deterministic A-star
  routing, diagonals, occupied targets, queued movement, recalculation,
  no-path state, and transit conflict arbitration.
* **Battery and generation** - working - Rust owns node and hauler batteries,
  minimum capacity, clamped charge, overdraw rejection, capacity-weighted
  adjacent balancing, fuel burn, owner-specific system costs, and starvation.
* **Automatic power lines** - working - Rust owns one-time nearest-target
  selection, greedy eight-neighbor construction, pass-through line cells,
  1,000-capacity line batteries, snapshots, and viewer projection.
* **World mutation** - working core - Rust owns scenario creation, spawnable
  factory-output retrieval, build-site deployment, topology replacement,
  spawned-structure occupancy, source activation, and ordered deletion.
  Player-driven placement and general structure deletion remain.
* **Player controls and UI** - partial - Rust owns playback controls,
  scenario selection, snapshots, metrics, and activity presentation. Camera
  navigation, object selection, placement, and programming remain.
* **Observability** - working - Rust owns structured snapshots, event feeds,
  deterministic metrics, and headless JSONL output.

The agent updates this file when a remaining item becomes working, then removes
the superseded C# subsystem in the same or a following verified slice.
