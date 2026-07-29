# Unity feature audit

This audit maps every retained Unity gameplay subsystem to its Rust replacement
status. “Working” means the Rust path has deterministic tests and, where the
behavior is visual, a scenario exposed by the Bevy/Wasm viewer.

* **Content catalog** - working - `factory_content` owns all active resources,
  products, spawnable recipes, physical values, timing, multipliers, manifest
  flags, and spawn flags from `FactoryGameContent.cs`.
* **Resource containers** - working core - Rust owns capacity, weight, volume,
  reservations, insertion, removal, and transfer. Adjacent automated insertion
  and retrieval remain.
* **Mining** - working - Rust owns finite and manifest extraction, deployment
  requirements, power gating, depletion, and source/drill teardown.
* **Production** - working core - Rust owns multi-input recipes, progress,
  output capacity, and power gating. A general multi-factory chain remains.
* **Dispatch** - working core - Rust owns collect, deliver, retrieve, deploy,
  fleet arbitration, in-flight work, phase transitions, and fuel delivery.
  General object-type targeting and blocked-adjacency rejection remain.
* **Movement and pathfinding** - working - Rust owns deterministic A-star
  routing, diagonals, occupied targets, queued movement, recalculation,
  no-path state, and transit conflict arbitration.
* **Battery and generation** - working aggregate - Rust owns finite charge,
  fuel burn, system energy costs, and power starvation. Per-object battery
  balancing remains.
* **Automatic power lines** - not started - Rust still needs nearest-generator
  routing and line construction.
* **World mutation** - working core - Rust owns scenario creation,
  retrieve/deploy mutation, occupancy, and ordered deletion. General
  player-driven spawn, placement, and deletion remain.
* **Player controls and UI** - partial - Rust owns playback controls,
  scenario selection, snapshots, metrics, and activity presentation. Camera
  navigation, object selection, placement, and programming remain.
* **Observability** - working - Rust owns structured snapshots, event feeds,
  deterministic metrics, and headless JSONL output.

The agent updates this file when a remaining item becomes working, then removes
the superseded C# subsystem in the same or a following verified slice.
